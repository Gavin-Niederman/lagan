use std::{ffi::CString, future::Future, task::Poll};

use ntcore_sys::{
    NT_DisposeValueArray, NT_GetTopicCached, NT_GetTopicExists, NT_GetTopicPersistent, NT_GetTopicRetained, NT_GetTopicType, NT_GetTopicTypeString, NT_Publish, NT_Publisher, NT_ReadQueueValue, NT_Release, NT_SetBoolean, NT_SetBooleanArray, NT_SetDouble, NT_SetDoubleArray, NT_SetFloat, NT_SetFloatArray, NT_SetInteger, NT_SetIntegerArray, NT_SetRaw, NT_SetString, NT_SetStringArray, NT_SetTopicCached, NT_SetTopicPersistent, NT_SetTopicRetained, NT_Subscribe, NT_Subscriber, NT_Topic, WPI_String
};
use snafu::ensure;

use crate::{
    nt_types::{PubSubOptions, RawValue, Value, ValueFlags, ValueType}, Instance, InvalidTypeSnafu, NetworkTablesError, SetToUnassignedSnafu
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Topic<'a, I: Instance + ?Sized> {
    pub(crate) instance: &'a I,
    pub(crate) handle: NT_Topic,
    pub(crate) name: String,
}

impl<I: Instance + ?Sized> Topic<'_, I> {
    pub fn subscribe(&self, expected_type: ValueType, expected_type_string: impl AsRef<str>, options: PubSubOptions) -> TopicSubscriber<'_, I> {
        let type_str = CString::new(expected_type_string.as_ref()).unwrap();
        let raw_type_str = WPI_String::from(type_str.as_c_str());
        
        let raw_options = options.into();
        let handle = unsafe {
            NT_Subscribe(self.handle(), expected_type.into(), &raw const raw_type_str, &raw const raw_options)
        };

        TopicSubscriber {
            handle,
            topic: self,
        }
    }

    pub fn publish(&self, expected_type: ValueType, expected_type_string: impl AsRef<str>, options: PubSubOptions) -> TopicPublisher<'_, I> {
        let type_str = CString::new(expected_type_string.as_ref()).unwrap();
        let raw_type_str = WPI_String::from(type_str.as_c_str());
        
        let raw_options = options.into();
        let handle = unsafe {
            NT_Publish(self.handle(), expected_type.into(), &raw const raw_type_str, &raw const raw_options)
        };

        TopicPublisher {
            handle,
            topic: self,
        }
    }

    pub fn value_type(&self) -> ValueType {
        let raw_type = unsafe { NT_GetTopicType(self.handle()) };

        raw_type.into()
    }

    /// Returns the type of the topic as a string if the topic exists.
    /// This may contain more info than [`Self::value_type`] expecially when the type is [`NetworkTablesValueType::Raw`].
    ///
    /// # Returns
    ///
    /// Returns `None` if the topic doesn't exist. [`Self::is_active`]
    pub fn value_type_string(&self) -> Option<String> {
        if self.is_nonexistant() {
            return None;
        }

        let mut raw_string = unsafe { std::mem::zeroed() };
        unsafe {
            NT_GetTopicTypeString(self.handle(), &raw mut raw_string);
        }

        // Safety: NT should only return a nullptr when the topic does not exist.
        Some(
            String::from_utf8_lossy(unsafe {
                std::slice::from_raw_parts(raw_string.str.cast(), raw_string.len)
            })
            .into_owned(),
        )
    }

    pub fn set_flags(&self, flags: ValueFlags) {
        let persist = flags.contains(ValueFlags::PERSISTENT).into();
        let cache = (!flags.contains(ValueFlags::UNCACHED)).into();
        let retain = flags.contains(ValueFlags::RETAINED).into();

        unsafe {
            NT_SetTopicPersistent(self.handle(), persist);
            NT_SetTopicCached(self.handle(), cache);
            NT_SetTopicRetained(self.handle(), retain);
        }
    }

    pub fn flags(&self) -> ValueFlags {
        let (persist, cache, retain) = unsafe {
            (
                NT_GetTopicPersistent(self.handle()) == 1,
                NT_GetTopicCached(self.handle()) == 1,
                NT_GetTopicRetained(self.handle()) == 1,
            )
        };
        let mut flags = ValueFlags::empty();
        if persist {
            flags |= ValueFlags::PERSISTENT
        }
        if !cache {
            flags |= ValueFlags::UNCACHED
        }
        if retain {
            flags |= ValueFlags::RETAINED
        }

        flags
    }

    /// Returns true if the topic has at least one publisher
    pub fn is_existant(&self) -> bool {
        (unsafe { NT_GetTopicExists(self.handle()) } == 1)
    }
    /// Returns true if the topic has 0 publishers
    pub fn is_nonexistant(&self) -> bool {
        !self.is_existant()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// # Safety
    ///
    /// Caller must ensure that the returned handle is only used while the topic is valid.
    pub unsafe fn handle(&self) -> NT_Topic {
        self.handle
    }
}

impl<I: Instance + ?Sized> Drop for Topic<'_, I> {
    fn drop(&mut self) {
        unsafe {
            NT_Release(self.handle());
        }
    }
}

pub struct TopicSubscriberReadQueueRawFuture<'a, I: Instance + ?Sized> {
    subscriber: &'a TopicSubscriber<'a, I>,
}
impl<I: Instance + ?Sized> Future for TopicSubscriberReadQueueRawFuture<'_, I> {
    type Output = Vec<RawValue>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(values) = self.subscriber.try_read_update_queue_raw() {
            Poll::Ready(values)
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TopicSubscriber<'a, I: Instance + ?Sized> {
    handle: NT_Subscriber,
    topic: &'a Topic<'a, I>,
}

macro_rules! typed_reader {
    {$($ident:ident: $variant:ident => $ty:ty),*} => {
        $(
            /// Returns the value of this topic if it is of the specified type.
            /// Returns `None` if the type of the entry is not of the specified type.
            pub async fn $ident(&self) -> Option<$ty> {
                match self.value().await {
                    Value::$variant(v) => Some(v),
                    _ => None
                }
            }
        )*
    };
}

impl<I: Instance + ?Sized> TopicSubscriber<'_, I> {
    /// Returns all of the new topic values since the last read in their raw form (timestamps included).
    ///
    /// If there have been no new updates, None is returned.
    pub fn try_read_update_queue_raw(&self) -> Option<Vec<RawValue>> {
        let mut count = 0;
        let raw_values = unsafe { NT_ReadQueueValue(self.handle(), &raw mut count) };
        if count == 0 {
            return None;
        }

        let values = unsafe { std::slice::from_raw_parts(raw_values, count) };
        let values = values
            .iter()
            .map(|v| (*v).into())
            .collect::<Vec<RawValue>>();
        unsafe {
            NT_DisposeValueArray(raw_values, count);
        }

        Some(values)
    }

    pub fn try_read_update_queue(&self) -> Option<Vec<Value>> {
        let values = self.try_read_update_queue_raw()?;
        Some(values.into_iter().map(|v| v.data).collect())
    }

    pub fn update_queue_raw(&self) -> TopicSubscriberReadQueueRawFuture<'_, I> {
        TopicSubscriberReadQueueRawFuture { subscriber: self }
    }
    pub async fn update_queue(&self) -> Vec<Value> {
        let values = self.update_queue_raw().await;
        values.into_iter().map(|v| v.data).collect()
    }

    pub async fn value(&self) -> Value {
        let updates = self.update_queue().await;
        updates.last().unwrap().clone()
    }

    typed_reader!{
        value_bool: Bool => bool,
        value_i64: I64 => i64,
        value_f32: F32 => f32,
        value_f64: F64 => f64,
        value_string: String => String,
        value_raw: Raw => Vec<u8>,
        value_bool_array: BoolArray => Vec<bool>,
        value_f64_array: F64Array => Vec<f64>,
        value_f32_array: F32Array => Vec<f32>,
        value_i64_array: I64Array => Vec<i64>,
        value_string_array: StringArray => Vec<String>
    }

    /// # Safety
    ///
    /// Caller must ensure that the returned handle is only used while the topic and subscriber is valid.
    pub unsafe fn handle(&self) -> NT_Subscriber {
        self.handle
    }
}

impl<I: Instance + ?Sized> Drop for TopicSubscriber<'_, I> {
    fn drop(&mut self) {
        unsafe {
            NT_Release(self.handle());
        }
    }
}


#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TopicPublisher<'a, I: Instance + ?Sized> {
    handle: NT_Publisher,
    topic: &'a Topic<'a, I>,
}


macro_rules! typed_setter {
    {$($ident:ident: $ty:ty => $variant:ident),*} => {
        $(
            /// Sets the value of this topic if it is of the specified type.
            pub fn $ident(&self, value: $ty) -> Result<(), NetworkTablesError> {
                self.set_value(Value::$variant(value))
            }
        )*
    };
}

impl<I: Instance + ?Sized> TopicPublisher<'_, I> {
    pub fn set_value(&self, value: Value) -> Result<(), NetworkTablesError> {
        ensure!(value.value_type() == self.topic.value_type(), InvalidTypeSnafu {
            current_type: self.topic.value_type(),
            given_type: value.value_type(),
        });

        macro_rules! set_simple_array {
            ($name:ident($field:ident)) => {{
                let len = $field.len() as _;
                let raw = $field.as_ptr();
                unsafe { $name(self.handle(), 0, raw, len) }
            }};
        }

        // ntcore only has typed topic publisher setters so we have to match
        let result = match value {
            Value::Unassigned => return SetToUnassignedSnafu.fail(),
            Value::Bool(value) => {
                unsafe {
                    NT_SetBoolean(self.handle(), 0, value.into())
                }
            },
            Value::I64(value) => {
                unsafe {
                    NT_SetInteger(self.handle(), 0, value)
                }
            },
            Value::F32(value) => {
                unsafe {
                    NT_SetFloat(self.handle(), 0, value)
                }
            },
            Value::F64(value) => {
                unsafe {
                    NT_SetDouble(self.handle(), 0, value)
                }
            },
            Value::String(string) => {
                let string = CString::new(string).unwrap();
                let wpi_string = WPI_String::from(string.as_c_str());
                unsafe {
                    NT_SetString(self.handle(), 0, &raw const wpi_string)
                }
            },
            Value::Raw(value) => set_simple_array!(NT_SetRaw(value)),
            Value::F64Array(value) => set_simple_array!(NT_SetDoubleArray(value)),
            Value::F32Array(value) => set_simple_array!(NT_SetFloatArray(value)),
            Value::I64Array(value) => set_simple_array!(NT_SetIntegerArray(value)),
            Value::BoolArray(value) => {
                let bools = value.into_iter().map(|b| b.into()).collect::<Vec<_>>();
                let len = bools.len() as _;
                let raw = bools.as_ptr();
                unsafe { NT_SetBooleanArray(self.handle(), 0, raw, len) }
            },
            Value::StringArray(value) => {
                let c_strings = value
                    .into_iter()
                    .map(|s| CString::new(s).unwrap())
                    .collect::<Vec<_>>();
                let wpi_strings = c_strings
                    .iter()
                    .map(|s| WPI_String::from(s.as_c_str()))
                    .collect::<Vec<_>>();
                let len = wpi_strings.len() as _;
                let raw = wpi_strings.as_ptr();
                unsafe { NT_SetStringArray(self.handle(), 0, raw, len) }
            },
        } == 1;

        debug_assert!(result);

        Ok(())
    }

    typed_setter! {
        set_value_bool: bool => Bool,
        set_value_i64: i64 => I64,
        set_value_f32: f32 => F32,
        set_value_f64: f64 => F64,
        set_value_string: String => String,
        set_value_raw: Vec<u8> => Raw,
        set_value_bool_array: Vec<bool> => BoolArray,
        set_value_f64_array: Vec<f64> => F64Array,
        set_value_f32_array: Vec<f32> => F32Array,
        set_value_i64_array: Vec<i64> => I64Array,
        set_value_string_array: Vec<String> => StringArray
    }

    /// # Safety
    ///
    /// Caller must ensure that the returned handle is only used while the topic and publisher is valid.
    pub unsafe fn handle(&self) -> NT_Subscriber {
        self.handle
    }
}

impl<I: Instance + ?Sized> Drop for TopicPublisher<'_, I> {
    fn drop(&mut self) {
        unsafe {
            NT_Release(self.handle());
        }
    }
}
