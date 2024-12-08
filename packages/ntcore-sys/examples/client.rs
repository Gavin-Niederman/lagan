use std::{thread::sleep, time::Duration, u32};

use ntcore_sys::{
    NT_AddLogger, NT_Event, NT_GetDefaultInstance, NT_GetEntry, NT_GetEntryValue, NT_IsConnected,
    NT_SetServer, NT_StartClient4, NT_Type, NT_Value, WPI_String,
};

extern "C" fn log_cb(_data: *mut std::ffi::c_void, event: *const NT_Event) {
    let message = unsafe { std::ffi::CStr::from_ptr((*event).data.logMessage.message.str) };
    // println!("Log: {}", message.to_string_lossy());
}

fn main() {
    unsafe {
        let inst = NT_GetDefaultInstance();
        let mut data = ();
        NT_AddLogger(inst, 0, u32::MAX, (&raw mut data).cast(), log_cb);

        let mut identity: WPI_String = c"127.0.0.1".into();
        NT_StartClient4(inst, &raw mut identity);
        let mut server_name: WPI_String = c"localhost".into();
        NT_SetServer(inst, &raw mut server_name, 5810);
        // Who knows
        sleep(Duration::from_secs(2));
        println!("Connected? {:?}", NT_IsConnected(inst));

        let mut name: WPI_String = c"/foo".into();
        let foo = NT_GetEntry(inst, &raw mut name);
        let mut foo_val = std::mem::zeroed::<NT_Value>();

        loop {
            NT_GetEntryValue(foo, &raw mut foo_val);
            if foo_val.r#type == NT_Type::NT_DOUBLE {
                println!("Foo: {}", foo_val.data.v_double);
            } else {
                println!("Foo: not a double {:?}", foo_val.r#type);
            }
            sleep(Duration::from_millis(100));
        }
    }
}
