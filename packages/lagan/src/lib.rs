use std::{ffi::CString, fmt::Debug};

use log::{log, Level};
use nt_types::{
    NetworkTablesEntryFlags, NetworkTablesRawValue, NetworkTablesValue, NetworkTablesValueType,
};
use ntcore_sys::{
    NT_Entry, NT_EntryFlags, NT_Event, NT_GetEntry, NT_GetEntryType, NT_GetEntryValue, NT_Inst,
    NT_LogLevel, NT_LogMessage, NT_Now, NT_SetEntryFlags, NT_SetEntryValue, NT_Value, NT_ValueData,
    NT_ValueDataArray, NT_ValueDataRaw, WPI_String,
};
use snafu::Snafu;

pub mod client;
pub mod nt_types;
pub mod server;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NetworkTablesVersion {
    #[default]
    V4,
    V3,
}

/// # Safety
///
/// Caller must ensure that this function is only used as a listener callback for a logger.
pub unsafe extern "C" fn default_log_callback(
    _data: *mut std::ffi::c_void,
    message: *const NT_Event,
) {
    let message = unsafe { (*message).data.logMessage };
    log_callback_inner(message);
}

fn log_callback_inner(message: NT_LogMessage) {
    let level = if message.level >= NT_LogLevel::NT_LOG_ERROR.bits() {
        Level::Error
    } else if message.level >= NT_LogLevel::NT_LOG_WARNING.bits() {
        Level::Warn
    } else if message.level >= NT_LogLevel::NT_LOG_INFO.bits() {
        Level::Info
    } else if message.level >= NT_LogLevel::NT_LOG_DEBUG2.bits() {
        Level::Debug
    } else if message.level >= NT_LogLevel::NT_LOG_DEBUG3.bits() {
        Level::Trace
    } else {
        return;
    };

    let file = String::from_utf8_lossy(unsafe {
        std::slice::from_raw_parts::<u8>(message.filename.str.cast(), message.filename.len)
    })
    .into_owned();
    let message_text = String::from_utf8_lossy(unsafe {
        std::slice::from_raw_parts::<u8>(message.message.str.cast(), message.message.len)
    })
    .into_owned();

    match level {
        Level::Error | Level::Warn | Level::Trace => {
            log!(level, "{}:{}: {}", file, message.line, message_text)
        }
        Level::Info | Level::Debug => log!(level, "{}", message_text),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkTablesEntry<'a, I: Instance + ?Sized> {
    instance: &'a I,
    handle: NT_Entry,
    name: String,
}

macro_rules! typed_value_getter {
    {$($ident:ident: $variant:ident => $ty:ty),*} => {
        $(
            /// Returns the value of this entry if it is of the specified type.
            /// Returns `None` if the type of the entry is not of the specified type.
            pub fn $ident(&self) -> Option<$ty> {
                match self.value() {
                    NetworkTablesValue::$variant(value) => Some(value),
                    _ => None,
                }
            }
        )*
    };
}

impl<I: Instance + ?Sized> NetworkTablesEntry<'_, I> {
    pub fn value(&self) -> NetworkTablesValue {
        self.raw_value().data
    }

    typed_value_getter! {
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

    pub fn value_type(&self) -> NetworkTablesValueType {
        unsafe { NT_GetEntryType(self.handle()) }.into()
    }

    pub fn set_value(&self, value: NetworkTablesValue) -> Result<(), NetworkTablesEntryError> {
        let current_value = self.raw_value();
        let current_type = current_value.data.value_type();

        if current_type != NetworkTablesValueType::Unassigned && current_type != value.value_type() {
            return Err(NetworkTablesEntryError::InvalidType {
                current_type,
                given_type: value.value_type(),
            });
        }

        let timestamp = unsafe { NT_Now() };
        let mut new_value = NT_Value {
            r#type: value.value_type().into(),
            last_change: timestamp,
            server_time: current_value.server_time.as_micros() as _,
            data: unsafe { std::mem::zeroed() },
        };
        if self.instance.is_server() {
            new_value.server_time = timestamp;
        }

        //Safety: This raw data cannot be used after the values it points to are dropped.
        //Safety: for this reason, the types boolean array and string array have to be used inside the match arms
        //Safety: because they need to be converted to nt types.
        let raw_value_data = match value {
            NetworkTablesValue::Unassigned => todo!(),
            NetworkTablesValue::Bool(value) => NT_ValueData {
                v_boolean: value as _,
            },
            NetworkTablesValue::I64(value) => NT_ValueData { v_int: value },
            NetworkTablesValue::F32(value) => NT_ValueData { v_float: value },
            NetworkTablesValue::F64(value) => NT_ValueData { v_double: value },
            NetworkTablesValue::String(value) => NT_ValueData {
                v_string: WPI_String {
                    str: value.as_ptr().cast(),
                    len: value.len() as _,
                },
            },
            NetworkTablesValue::Raw(value) => NT_ValueData {
                v_raw: NT_ValueDataRaw {
                    data: value.as_ptr(),
                    size: value.len() as _,
                },
            },
            NetworkTablesValue::F64Array(value) => NT_ValueData {
                arr_double: NT_ValueDataArray {
                    arr: value.as_ptr(),
                    size: value.len() as _,
                },
            },
            NetworkTablesValue::F32Array(value) => NT_ValueData {
                arr_float: NT_ValueDataArray {
                    arr: value.as_ptr(),
                    size: value.len() as _,
                },
            },
            NetworkTablesValue::I64Array(value) => NT_ValueData {
                arr_int: NT_ValueDataArray {
                    arr: value.as_ptr(),
                    size: value.len() as _,
                },
            },
            NetworkTablesValue::BoolArray(value) => todo!(),
            NetworkTablesValue::StringArray(value) => todo!(),
        };

        new_value.data = raw_value_data;

        let status = unsafe { NT_SetEntryValue(self.handle(), &raw const new_value) };
        debug_assert_eq!(status, 1);

        Ok(())
    }

    pub fn set_flags(&self, flags: NetworkTablesEntryFlags) {
        unsafe {
            NT_SetEntryFlags(self.handle(), NT_EntryFlags::from_bits_retain(flags.bits()));
        }
    }

    pub fn is_assigned(&self) -> bool {
        !matches!(self.value_type(), NetworkTablesValueType::Unassigned)
    }
    pub fn is_unassigned(&self) -> bool {
        !self.is_assigned()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn raw_value(&self) -> NetworkTablesRawValue {
        let mut raw_value = unsafe { std::mem::zeroed() };
        unsafe {
            NT_GetEntryValue(self.handle(), &raw mut raw_value);
        }
        raw_value.into()
    }

    /// # Safety
    ///
    /// Caller must ensure that the returned handle is only used while the table entry is valid.
    pub unsafe fn handle(&self) -> NT_Entry {
        self.handle
    }
}

/// Errors that can occur when interacting with a NetworkTables entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Snafu)]
pub enum NetworkTablesEntryError {
    /// Attempted to set an entry to a value of a different type than it currently is.
    #[snafu(display("Attempted to set an entry to a value of type {given_type:?} while it was of type {current_type:?}."))]
    InvalidType {
        current_type: NetworkTablesValueType,
        given_type: NetworkTablesValueType,
    },
}

pub trait Instance {
    fn entry(&self, name: impl AsRef<str>) -> NetworkTablesEntry<'_, Self> {
        let raw_name = CString::new(name.as_ref()).unwrap();
        let raw_name = WPI_String::from(raw_name.as_c_str());

        let handle = unsafe { NT_GetEntry(self.handle(), &raw const raw_name) };

        NetworkTablesEntry {
            instance: self,
            handle,
            name: name.as_ref().to_owned(),
        }
    }

    fn is_server(&self) -> bool;
    fn is_client(&self) -> bool {
        !self.is_server()
    }

    /// # Safety
    ///
    /// Caller must ensure that the returned handle is only used while the instance is valid.
    unsafe fn handle(&self) -> NT_Inst;
}
