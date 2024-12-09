use std::{
    ops::{Add, AddAssign, Sub, SubAssign},
    slice,
    time::Duration,
};

use bitflags::bitflags;
use ntcore_sys::{NT_Now, NT_Type, NT_Value};

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
pub enum NetworkTablesValueType {
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
impl From<NT_Type> for NetworkTablesValueType {
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
impl From<NetworkTablesValueType> for NT_Type {
    fn from(value: NetworkTablesValueType) -> Self {
        match value {
            NetworkTablesValueType::Unassigned => NT_Type::NT_UNASSIGNED,
            NetworkTablesValueType::Bool => NT_Type::NT_BOOLEAN,
            NetworkTablesValueType::I64 => NT_Type::NT_INTEGER,
            NetworkTablesValueType::F32 => NT_Type::NT_FLOAT,
            NetworkTablesValueType::F64 => NT_Type::NT_DOUBLE,
            NetworkTablesValueType::String => NT_Type::NT_STRING,
            NetworkTablesValueType::Raw => NT_Type::NT_RAW,
            NetworkTablesValueType::BoolArray => NT_Type::NT_BOOLEAN_ARRAY,
            NetworkTablesValueType::F64Array => NT_Type::NT_DOUBLE_ARRAY,
            NetworkTablesValueType::F32Array => NT_Type::NT_FLOAT_ARRAY,
            NetworkTablesValueType::I64Array => NT_Type::NT_INTEGER_ARRAY,
            NetworkTablesValueType::StringArray => NT_Type::NT_STRING_ARRAY,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkTablesValue {
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
impl NetworkTablesValue {
    pub fn value_type(&self) -> NetworkTablesValueType {
        match self {
            Self::Unassigned => NetworkTablesValueType::Unassigned,
            Self::Bool(_) => NetworkTablesValueType::Bool,
            Self::I64(_) => NetworkTablesValueType::I64,
            Self::F32(_) => NetworkTablesValueType::F32,
            Self::F64(_) => NetworkTablesValueType::F64,
            Self::String(_) => NetworkTablesValueType::String,
            Self::Raw(_) => NetworkTablesValueType::Raw,
            Self::BoolArray(_) => NetworkTablesValueType::BoolArray,
            Self::F64Array(_) => NetworkTablesValueType::F64Array,
            Self::F32Array(_) => NetworkTablesValueType::F32Array,
            Self::I64Array(_) => NetworkTablesValueType::I64Array,
            Self::StringArray(_) => NetworkTablesValueType::StringArray,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkTablesRawValue {
    pub data: NetworkTablesValue,
    pub last_change: NetworkTablesInstant,
    pub server_time: NetworkTablesInstant,
}

impl From<NT_Value> for NetworkTablesRawValue {
    // Oh boy, this is going to be a fun one
    fn from(value: NT_Value) -> Self {
        let last_change = NetworkTablesInstant::from_micros(value.last_change as _);
        let server_time = NetworkTablesInstant::from_micros(value.server_time as _);
        let data = match value.r#type {
            NT_Type::NT_UNASSIGNED | NT_Type::NT_RPC => NetworkTablesValue::Unassigned,
            NT_Type::NT_BOOLEAN => NetworkTablesValue::Bool(unsafe { value.data.v_boolean == 1 }),
            NT_Type::NT_INTEGER => NetworkTablesValue::I64(unsafe { value.data.v_int }),
            NT_Type::NT_FLOAT => NetworkTablesValue::F32(unsafe { value.data.v_float }),
            NT_Type::NT_DOUBLE => NetworkTablesValue::F64(unsafe { value.data.v_double }),
            NT_Type::NT_STRING => {
                let string = unsafe {
                    String::from_utf8_lossy(slice::from_raw_parts(
                        value.data.v_string.str.cast(),
                        value.data.v_string.len,
                    ))
                }
                .into_owned();
                NetworkTablesValue::String(string)
            }
            NT_Type::NT_RAW => {
                let data = unsafe {
                    slice::from_raw_parts(value.data.v_raw.arr, value.data.v_raw.size as _)
                }
                .to_vec();
                NetworkTablesValue::Raw(data)
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
                NetworkTablesValue::BoolArray(data)
            }
            NT_Type::NT_DOUBLE_ARRAY => {
                let data = unsafe {
                    slice::from_raw_parts(
                        value.data.arr_double.arr,
                        value.data.arr_double.size as _,
                    )
                }
                .to_vec();
                NetworkTablesValue::F64Array(data)
            }
            NT_Type::NT_FLOAT_ARRAY => {
                let data = unsafe {
                    slice::from_raw_parts(value.data.arr_float.arr, value.data.arr_float.size as _)
                }
                .to_vec();
                NetworkTablesValue::F32Array(data)
            }
            NT_Type::NT_INTEGER_ARRAY => {
                let data = unsafe {
                    slice::from_raw_parts(value.data.arr_int.arr, value.data.arr_int.size as _)
                }
                .to_vec();
                NetworkTablesValue::I64Array(data)
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
                NetworkTablesValue::StringArray(data)
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
    pub struct NetworkTablesEntryFlags: u32 {
        const PERSISTENT = 1;
        const RETAINED = 2;
        const UNCACHED = 4;
    }
}
