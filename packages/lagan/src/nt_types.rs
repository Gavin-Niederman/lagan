use std::{
    ops::{Add, AddAssign, Sub, SubAssign},
    slice,
    time::Duration,
};

use bitflags::bitflags;
use ntcore_sys::{NT_Now, NT_PubSubOptions, NT_Type, NT_Value};
use typed_builder::TypedBuilder;

/// A monotonic clock timestamp that is used to timestamp network tables values.
/// Instants have microsecond precision.
///
/// This API matches the [`std::time::Instant`] API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NetworkTablesInstant {
    micros: u64,
}
impl NetworkTablesInstant {
    pub fn now() -> Self {
        let micros = unsafe { NT_Now() } as _;
        Self { micros }
    }

    pub(crate) fn from_micros(micros: u64) -> Self {
        Self { micros }
    }
    pub(crate) fn as_micros(&self) -> u64 {
        self.micros
    }

    pub fn elapsed(&self) -> Duration {
        self.duration_since(Self::now())
    }
    pub fn duration_since(&self, earlier: Self) -> Duration {
        self.checked_duration_since(earlier).unwrap()
    }

    pub fn checked_add(&self, duration: std::time::Duration) -> Option<Self> {
        let micros = self.micros.checked_add(duration.as_micros() as u64)?;
        Some(Self { micros })
    }
    pub fn checked_sub(&self, duration: std::time::Duration) -> Option<Self> {
        let micros = self.micros.checked_sub(duration.as_micros() as u64)?;
        Some(Self { micros })
    }
    pub fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
        let micros = self.micros.checked_sub(earlier.micros)?;
        Some(std::time::Duration::from_micros(micros))
    }
    pub fn saturating_duration_since(&self, earlier: Self) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }
}

impl Add<Duration> for NetworkTablesInstant {
    type Output = Self;
    fn add(self, duration: Duration) -> Self {
        self.checked_add(duration).unwrap()
    }
}
impl AddAssign<Duration> for NetworkTablesInstant {
    fn add_assign(&mut self, duration: Duration) {
        *self = *self + duration;
    }
}

impl Sub for NetworkTablesInstant {
    type Output = Duration;
    fn sub(self, other: Self) -> Duration {
        self.saturating_duration_since(other)
    }
}
impl Sub<Duration> for NetworkTablesInstant {
    type Output = Self;
    fn sub(self, duration: Duration) -> Self {
        self.checked_sub(duration).unwrap()
    }
}
impl SubAssign<Duration> for NetworkTablesInstant {
    fn sub_assign(&mut self, duration: Duration) {
        *self = *self - duration;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    Unassigned,
    Bool,
    I64,
    F32,
    F64,
    String,
    Raw,
    BoolArray,
    F64Array,
    F32Array,
    I64Array,
    StringArray,
}
impl From<NT_Type> for ValueType {
    fn from(value: NT_Type) -> Self {
        match value {
            NT_Type::NT_UNASSIGNED | NT_Type::NT_RPC => Self::Unassigned,
            NT_Type::NT_BOOLEAN => Self::Bool,
            NT_Type::NT_INTEGER => Self::I64,
            NT_Type::NT_FLOAT => Self::F32,
            NT_Type::NT_DOUBLE => Self::F64,
            NT_Type::NT_STRING => Self::String,
            NT_Type::NT_RAW => Self::Raw,
            NT_Type::NT_BOOLEAN_ARRAY => Self::BoolArray,
            NT_Type::NT_DOUBLE_ARRAY => Self::F64Array,
            NT_Type::NT_FLOAT_ARRAY => Self::F32Array,
            NT_Type::NT_INTEGER_ARRAY => Self::I64Array,
            NT_Type::NT_STRING_ARRAY => Self::StringArray,
            _ => unreachable!("Invalid NT_Type"),
        }
    }
}
impl From<ValueType> for NT_Type {
    fn from(value: ValueType) -> Self {
        match value {
            ValueType::Unassigned => NT_Type::NT_UNASSIGNED,
            ValueType::Bool => NT_Type::NT_BOOLEAN,
            ValueType::I64 => NT_Type::NT_INTEGER,
            ValueType::F32 => NT_Type::NT_FLOAT,
            ValueType::F64 => NT_Type::NT_DOUBLE,
            ValueType::String => NT_Type::NT_STRING,
            ValueType::Raw => NT_Type::NT_RAW,
            ValueType::BoolArray => NT_Type::NT_BOOLEAN_ARRAY,
            ValueType::F64Array => NT_Type::NT_DOUBLE_ARRAY,
            ValueType::F32Array => NT_Type::NT_FLOAT_ARRAY,
            ValueType::I64Array => NT_Type::NT_INTEGER_ARRAY,
            ValueType::StringArray => NT_Type::NT_STRING_ARRAY,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unassigned,
    Bool(bool),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Raw(Vec<u8>),
    BoolArray(Vec<bool>),
    F64Array(Vec<f64>),
    F32Array(Vec<f32>),
    I64Array(Vec<i64>),
    StringArray(Vec<String>),
}
impl Value {
    pub fn value_type(&self) -> ValueType {
        match self {
            Self::Unassigned => ValueType::Unassigned,
            Self::Bool(_) => ValueType::Bool,
            Self::I64(_) => ValueType::I64,
            Self::F32(_) => ValueType::F32,
            Self::F64(_) => ValueType::F64,
            Self::String(_) => ValueType::String,
            Self::Raw(_) => ValueType::Raw,
            Self::BoolArray(_) => ValueType::BoolArray,
            Self::F64Array(_) => ValueType::F64Array,
            Self::F32Array(_) => ValueType::F32Array,
            Self::I64Array(_) => ValueType::I64Array,
            Self::StringArray(_) => ValueType::StringArray,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawValue {
    pub data: Value,
    pub last_change: NetworkTablesInstant,
    pub server_time: NetworkTablesInstant,
}

impl From<NT_Value> for RawValue {
    // Oh boy, this is going to be a fun one
    fn from(value: NT_Value) -> Self {
        let last_change = NetworkTablesInstant::from_micros(value.last_change as _);
        let server_time = NetworkTablesInstant::from_micros(value.server_time as _);
        let data = match value.r#type {
            NT_Type::NT_UNASSIGNED | NT_Type::NT_RPC => Value::Unassigned,
            NT_Type::NT_BOOLEAN => Value::Bool(unsafe { value.data.v_boolean == 1 }),
            NT_Type::NT_INTEGER => Value::I64(unsafe { value.data.v_int }),
            NT_Type::NT_FLOAT => Value::F32(unsafe { value.data.v_float }),
            NT_Type::NT_DOUBLE => Value::F64(unsafe { value.data.v_double }),
            NT_Type::NT_STRING => {
                let string = unsafe {
                    String::from_utf8_lossy(slice::from_raw_parts(
                        value.data.v_string.str.cast(),
                        value.data.v_string.len,
                    ))
                }
                .into_owned();
                Value::String(string)
            }
            NT_Type::NT_RAW => {
                let data = unsafe {
                    slice::from_raw_parts(value.data.v_raw.arr, value.data.v_raw.size as _)
                }
                .to_vec();
                Value::Raw(data)
            }
            NT_Type::NT_BOOLEAN_ARRAY => {
                let data = unsafe {
                    slice::from_raw_parts(
                        value.data.arr_boolean.arr,
                        value.data.arr_boolean.size as _,
                    )
                }
                .iter()
                .map(|b| *b == 1)
                .collect::<Vec<_>>();
                Value::BoolArray(data)
            }
            NT_Type::NT_DOUBLE_ARRAY => {
                let data = unsafe {
                    slice::from_raw_parts(
                        value.data.arr_double.arr,
                        value.data.arr_double.size as _,
                    )
                }
                .to_vec();
                Value::F64Array(data)
            }
            NT_Type::NT_FLOAT_ARRAY => {
                let data = unsafe {
                    slice::from_raw_parts(value.data.arr_float.arr, value.data.arr_float.size as _)
                }
                .to_vec();
                Value::F32Array(data)
            }
            NT_Type::NT_INTEGER_ARRAY => {
                let data = unsafe {
                    slice::from_raw_parts(value.data.arr_int.arr, value.data.arr_int.size as _)
                }
                .to_vec();
                Value::I64Array(data)
            }
            NT_Type::NT_STRING_ARRAY => {
                let data = unsafe {
                    slice::from_raw_parts(
                        value.data.arr_string.arr,
                        value.data.arr_string.size as _,
                    )
                }
                .iter()
                .map(|s| {
                    unsafe { String::from_utf8_lossy(slice::from_raw_parts(s.str.cast(), s.len)) }
                        .into_owned()
                })
                .collect::<Vec<_>>();
                Value::StringArray(data)
            }
            _ => unreachable!("Invalid NT_Type"),
        };

        Self {
            last_change,
            server_time,
            data,
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueFlags: u32 {
        const PERSISTENT = 1;
        const RETAINED = 2;
        const UNCACHED = 4;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TypedBuilder)]
pub struct PubSubOptions {
    /// Defaults to 1 if [`Self::send_all_updates`] is true, 20 otherwise.
    #[builder(default = None, setter(strip_option))]
    pub queue_length: Option<u32>,
    /// How frequently changes should be sent over the network.
    #[builder(default = Duration::from_millis(100))]
    pub update_interval: Duration,
    /// Send all value changes over the network
    #[builder(default)]
    pub send_all_updates: bool,
    /// If true, duplicate value changes will be ignored.
    #[builder(default = true)]
    pub ignore_duplicates: bool,
}
impl Default for PubSubOptions {
    fn default() -> Self {
        Self::builder().build()
    }
}
impl PubSubOptions {
    pub fn new(queue_length: Option<u32>, update_interval: Duration, send_all_updates: bool, ignore_duplicates: bool) -> Self {
        Self {
            queue_length,
            update_interval,
            send_all_updates,
            ignore_duplicates,
        }
    }
}

impl From<PubSubOptions> for NT_PubSubOptions {
    fn from(options: PubSubOptions) -> Self {
        let queue_length = options.queue_length.unwrap_or(0);
        let update_interval = options.update_interval.as_secs_f64();
        let send_all_updates = options.send_all_updates.into();
        let keep_duplicates = (!options.ignore_duplicates).into();

        Self {
            structSize: std::mem::size_of::<NT_PubSubOptions>() as _,
            pollStorage: queue_length,
            periodic: update_interval,
            excludePublisher: 0,
            sendAll: send_all_updates,
            topicsOnly: 0,
            prefixMatch: 0,
            keepDuplicates: keep_duplicates,
            disableRemote: 0,
            disableLocal: 0,
            excludeSelf: 0,
            hidden: 0,
        }
    }
}
impl From<NT_PubSubOptions> for PubSubOptions {
    fn from(options: NT_PubSubOptions) -> Self {
        Self {
            queue_length: Some(options.pollStorage),
            update_interval: Duration::from_secs_f64(options.periodic),
            send_all_updates: options.sendAll != 0,
            ignore_duplicates: options.keepDuplicates == 0,
        }
    }
}