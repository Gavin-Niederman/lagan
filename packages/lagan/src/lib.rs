use std::{ffi::CString, fmt::Debug};

use entry::NetworkTablesEntry;
use log::{log, Level};
use nt_types::NetworkTablesValue;
use ntcore_sys::{
    NT_Event, NT_GetEntry, NT_GetTopic, NT_Inst, NT_LogLevel, NT_LogMessage, WPI_String
};
use topic::NetworkTablesTopic;

pub mod client;
pub mod nt_types;
pub mod server;
pub mod entry;
pub mod topic;

pub mod prelude {
    pub use crate::{
        client::Client,
        server::Server,
        nt_types::{
            NetworkTablesValue,
            NetworkTablesEntryFlags,
        },
        Instance,
        NetworkTablesVersion
    };
}

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

    fn topic(&self, name: impl AsRef<str>) -> NetworkTablesTopic<'_, Self> {
        let raw_name = CString::new(name.as_ref()).unwrap();
        let raw_name = WPI_String::from(raw_name.as_c_str());

        let handle = unsafe { NT_GetTopic(self.handle(), &raw const raw_name) };

        NetworkTablesTopic {
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
