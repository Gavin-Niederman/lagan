use std::{ffi::CString, fmt::Debug};

use log::{log, Level};
use nt_types::{NetworkTablesRawValue, NetworkTablesValue, NetworkTablesValueType};
use ntcore_sys::{
    NT_Entry, NT_Event, NT_GetEntry, NT_GetEntryType, NT_GetEntryValue, NT_Inst, NT_LogLevel,
    NT_LogMessage, WPI_String,
};

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

    pub fn raw_value(&self) -> NetworkTablesRawValue {
        let mut raw_value = unsafe { std::mem::zeroed() };
        unsafe {
            NT_GetEntryValue(self.handle(), &raw mut raw_value);
        }
        raw_value.into()
    }

    pub fn is_assigned(&self) -> bool {
        !matches!(self.value_type(), NetworkTablesValueType::Unassigned)
    }
    pub fn is_unassigned(&self) -> bool {
        !self.is_assigned()
    }

    pub fn value_type(&self) -> NetworkTablesValueType {
        unsafe { NT_GetEntryType(self.handle()) }.into()
    }

    /// # Safety
    ///
    /// Caller must ensure that the returned handle is only used while the table entry is valid.
    pub unsafe fn handle(&self) -> NT_Entry {
        self.handle
    }
}

pub trait Instance {
    fn entry(&self, name: impl AsRef<str>) -> NetworkTablesEntry<'_, Self> {
        let name = CString::new(name.as_ref()).unwrap();
        let name = WPI_String::from(name.as_c_str());

        let handle = unsafe { NT_GetEntry(self.handle(), &raw const name) };

        NetworkTablesEntry {
            instance: self,
            handle,
        }
    }

    /// # Safety
    ///
    /// Caller must ensure that the returned handle is only used while the instance is valid.
    unsafe fn handle(&self) -> NT_Inst;
}
