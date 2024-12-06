use std::{
    ffi::{CString},
    net::SocketAddr,
};

use libntcore::{
    NT_GetDefaultInstance, NT_Inst, NT_SetServer, NT_StartClient3, NT_StartClient4, WPI_String,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NetworkTablesVersion {
    #[default]
    V4,
    V3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Client {
    instance: NT_Inst,
}

impl Client {
    pub fn new(
        version: NetworkTablesVersion,
        address: SocketAddr,
        name: Option<impl AsRef<str>>,
    ) -> Self {
        let instance = unsafe { NT_GetDefaultInstance() };
        unsafe {
            let identity = CString::new(address.ip().to_string()).unwrap();
            let identity = WPI_String::from(identity.as_c_str());
            match version {
                NetworkTablesVersion::V4 => NT_StartClient4(instance, &raw const identity),
                NetworkTablesVersion::V3 => NT_StartClient3(instance, &raw const identity),
            }

            let server_name = name
                .map(|name| CString::new(name.as_ref()).unwrap())
                .unwrap_or_else(|| CString::new("localhost").unwrap());
            let server_name = WPI_String::from(server_name.as_c_str());
            NT_SetServer(instance, &raw const server_name, address.port() as _);
        }

        Self { instance }
    }
}
