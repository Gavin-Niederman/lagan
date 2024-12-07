use std::{
    ops::{Add, AddAssign, Sub, SubAssign},
    slice,
    time::Duration,
};

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

pub enum NetworkTablesValueData {
    Unasigned,
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

pub struct NetworkTablesValue {
    pub data: NetworkTablesValueData,
    pub last_change: NetworkTablesInstant,
    pub server_time: NetworkTablesInstant,
}

impl From<NT_Value> for NetworkTablesValue {
    // Oh boy, this is going to be a fun one
    fn from(value: NT_Value) -> Self {
        let last_change = NetworkTablesInstant::from_micros(value.last_change as _);
        let server_time = NetworkTablesInstant::from_micros(value.server_time as _);
        let data = match value.r#type {
            NT_Type::NT_UNASSIGNED => NetworkTablesValueData::Unasigned,
            NT_Type::NT_BOOLEAN => {
                NetworkTablesValueData::Bool(unsafe { value.data.v_boolean == 1 })
            }
            NT_Type::NT_DOUBLE => NetworkTablesValueData::F64(unsafe { value.data.v_double }),
            NT_Type::NT_STRING => {
                let string = unsafe {
                    String::from_utf8_lossy(slice::from_raw_parts(
                        value.data.v_string.str.cast(),
                        value.data.v_string.len,
                    ))
                }
                .into_owned();
                NetworkTablesValueData::String(string)
            }
            NT_Type::NT_RAW => {
                let data = unsafe {
                    slice::from_raw_parts(value.data.v_raw.data, value.data.v_raw.size as _)
                }
                .to_vec();
                NetworkTablesValueData::Raw(data)
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
                NetworkTablesValueData::BoolArray(data)
            }
            NT_Type::NT_RPC => todo!(),
            _ => unreachable!("Invalid NT_Type"),
        };

        Self {
            last_change,
            server_time,
            data,
        }
    }
}
