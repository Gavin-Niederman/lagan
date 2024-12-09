use std::{future::Future, task::Poll};

use ntcore_sys::{
    NT_DisposeValueArray, NT_GetTopicCached, NT_GetTopicExists, NT_GetTopicPersistent,
    NT_GetTopicRetained, NT_GetTopicType, NT_GetTopicTypeString, NT_ReadQueueValue, NT_Release,
    NT_SetTopicCached, NT_SetTopicPersistent, NT_SetTopicRetained, NT_Subscriber, NT_Topic,
};

use crate::{
    nt_types::{RawValue, Value, ValueFlags, ValueType},
    Instance,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Topic<'a, I: Instance + ?Sized> {
    pub(crate) instance: &'a I,
    pub(crate) handle: NT_Topic,
    pub(crate) name: String,
}

impl<I: Instance + ?Sized> Topic<'_, I> {
    pub fn subscribe(&self, expected_type: ValueType, expected_type_string: impl AsRef<str>) -> TopicSubscriber<'_, I> {
        todo!()
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
        let exists = unsafe { NT_GetTopicExists(self.handle()) } == 1;
        exists
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
        if let Some(values) = self.subscriber.try_read_queue_raw() {
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

impl<I: Instance + ?Sized> TopicSubscriber<'_, I> {
    /// Returns all of the new entry values since the last read in their raw form (timestamps included).
    ///
    /// If there have been no new updates, None is returned.
    pub fn try_read_queue_raw(&self) -> Option<Vec<RawValue>> {
        let mut count = 0;
        let raw_values = unsafe { NT_ReadQueueValue(self.topic.handle(), &raw mut count) };
        if count == 0 {
            return None;
        }

        let values = unsafe { std::slice::from_raw_parts(raw_values, count) };
        let values = values
            .into_iter()
            .map(|v| (*v).into())
            .collect::<Vec<RawValue>>();
        unsafe {
            NT_DisposeValueArray(raw_values, count);
        }

        Some(values)
    }

    pub fn try_read_queue(&self) -> Option<Vec<Value>> {
        let values = self.try_read_queue_raw()?;
        Some(values.into_iter().map(|v| v.data).collect())
    }

    pub fn read_queue_raw(&self) -> TopicSubscriberReadQueueRawFuture<'_, I> {
        TopicSubscriberReadQueueRawFuture { subscriber: self }
    }
    pub async fn read_qeueu(&self) -> Vec<Value> {
        let values = self.read_queue_raw().await;
        values.into_iter().map(|v| v.data).collect()
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
