use std::{ffi::CString, net::SocketAddr};

use ntcore_sys::{
    NT_AddLogger, NT_CreateInstance, NT_DestroyInstance, NT_Inst, NT_SetServer,
    NT_StartClient3, NT_StartClient4, NT_StopClient, WPI_String,
};
use typed_builder::TypedBuilder;

use crate::{Instance, NetworkTablesVersion};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Client {
    instance: NT_Inst,
}

impl Client {
    pub fn new(
        version: NetworkTablesVersion,
        address: SocketAddr,
        server_name: Option<impl AsRef<str>>,
    ) -> Self {
        let instance = unsafe { NT_CreateInstance() };

        //TODO: Are these WPI_String pointers supposed to be static?
        //TODO: When can the identity and name safely be dropped?
        unsafe {
            NT_AddLogger(
                instance,
                0,
                u32::MAX,
                std::ptr::null_mut(),
                crate::default_log_callback,
            );

            let identity = CString::new(address.ip().to_string()).unwrap();
            let identity = WPI_String::from(identity.as_c_str());
            match version {
                NetworkTablesVersion::V4 => NT_StartClient4(instance, &raw const identity),
                NetworkTablesVersion::V3 => NT_StartClient3(instance, &raw const identity),
            }

            let server_name = server_name
                .map(|name| CString::new(name.as_ref()).unwrap())
                .unwrap_or_else(|| CString::new("").unwrap());
            let server_name = WPI_String::from(server_name.as_c_str());
            NT_SetServer(instance, &raw const server_name, address.port() as _);
        }

        Self { instance }
    }

    pub fn builder() -> ClientOptionsBuilder {
        ClientOptions::builder()
    }
}

impl Instance for Client {
    unsafe fn handle(&self) -> NT_Inst {
        self.instance
    }
    fn is_server(&self) -> bool {
        false
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe {
            NT_StopClient(self.instance);
            NT_DestroyInstance(self.instance);
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
#[builder(build_method(into = Client))]
pub struct ClientOptions {
    #[builder(default = None, setter(transform = |name: impl AsRef<str>| Some(name.as_ref().to_string())))]
    pub server_name: Option<String>,
    pub address: SocketAddr,
    pub version: NetworkTablesVersion,
}
impl From<ClientOptions> for Client {
    fn from(options: ClientOptions) -> Self {
        Client::new(options.version, options.address, options.server_name)
    }
}
