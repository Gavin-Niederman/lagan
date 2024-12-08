use std::{ffi::CString, net::SocketAddr};

use ntcore_sys::{
    NT_AddLogger, NT_DestroyInstance, NT_GetDefaultInstance, NT_Inst, NT_StartServer, NT_StopServer, WPI_String
};
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Server {
    instance: NT_Inst,
}

impl Server {
    /// Starts a new NetworkTables server.
    ///
    /// # Parameters
    ///
    /// - `persist_filename`: The name of the file to persist the server data to.
    ///   This file will be created in the current working directory.
    ///   Depending on the extension, the file will be saved in either JSON or INI format.
    /// - `listen_address`: The address to listen for incoming connections on.
    ///   If `None`, the server will listen on all addresses.
    /// - `nt3_port`: The port to listen for NetworkTables V3 clients on.
    /// - `nt4_port`: The port to listen for NetworkTables V4 clients on.
    pub fn new(
        persist_filename: impl AsRef<str>,
        listen_address: Option<SocketAddr>,
        nt3_port: u16,
        nt4_port: u16,
    ) -> Self {
        let instance = unsafe { NT_GetDefaultInstance() };

        //TODO: Are these WPI_String pointers supposed to be static?
        unsafe {
            NT_AddLogger(
                instance,
                0,
                u32::MAX,
                std::ptr::null_mut(),
                crate::default_log_callback,
            );

            let persist_filename = CString::new(persist_filename.as_ref()).unwrap();
            let persist_filename = WPI_String::from(persist_filename.as_c_str());

            let listen_address = listen_address.map(|address| {
                let address = CString::new(address.ip().to_string()).unwrap();
                WPI_String::from(address.as_c_str())
            });

            NT_StartServer(
                instance,
                &raw const persist_filename,
                listen_address
                    .map(|la| &raw const la)
                    .unwrap_or(std::ptr::null()),
                nt3_port as _,
                nt4_port as _,
            );
        }

        Self { instance }
    }

    pub fn builder() -> ServerOptionsBuilder {
        ServerOptions::builder()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        unsafe {
            NT_StopServer(self.instance);
            NT_DestroyInstance(self.instance);
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
#[builder(build_method(into = Server))]
pub struct ServerOptions {
    #[builder(setter(transform = |name: impl AsRef<str>| name.as_ref().to_string()))]
    pub persist_filename: String,
    #[builder(default = None, setter(strip_option))]
    pub listen_address: Option<SocketAddr>,
    #[builder(default = 1735)]
    pub nt3_port: u16,
    #[builder(default = 5810)]
    pub nt4_port: u16,
}
impl From<ServerOptions> for Server {
    fn from(options: ServerOptions) -> Self {
        Server::new(
            options.persist_filename,
            options.listen_address,
            options.nt3_port,
            options.nt4_port,
        )
    }
}
