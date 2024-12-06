use std::{thread::sleep, time::Duration, u32};

use ntcore_sys::{
    NT_AddLogger, NT_EntryFlags, NT_Event, NT_GetDefaultInstance, NT_GetEntry, NT_GetEntryValue, NT_SetEntryFlags, NT_SetEntryValue, NT_SetServer, NT_StartClient4, NT_StartServer, NT_Type, NT_Value, NT_ValueData, WPI_String
};

extern "C" fn log_cb(_data: *mut std::ffi::c_void, event: *const NT_Event) {
    let message = unsafe { std::ffi::CStr::from_ptr((*event).data.logMessage.message.str) };
    println!("Log: {}", message.to_string_lossy());
}

fn main() {
    unsafe {
        let inst = NT_GetDefaultInstance();
        let mut data = ();
        NT_AddLogger(inst, 0, u32::MAX, (&raw mut data).cast(), log_cb);

        let mut persist_name: WPI_String = "networktables.json".into();
        NT_StartServer(
            inst,
            &raw mut persist_name,
            0 as *const WPI_String,
            1735,
            5810,
        );

        // Who knows
        sleep(Duration::from_secs(1));

        let mut name: WPI_String = "/foo".into();
        let foo = NT_GetEntry(inst, &raw mut name);
        let val = NT_Value {
            r#type: NT_Type::NT_DOUBLE,
            last_change: 0,
            server_time: 0,
            data: NT_ValueData { v_double: 3.14 },
        };
        NT_SetEntryValue(foo, &raw const val);
        NT_SetEntryFlags(foo, NT_EntryFlags::NT_PERSISTENT.bits());
        loop {
            sleep(Duration::from_secs(1));
        }
    }
}
