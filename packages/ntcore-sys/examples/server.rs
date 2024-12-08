use std::{thread::sleep, time::Duration};

use ntcore_sys::{
    NT_AddLogger, NT_EntryFlags, NT_Event, NT_GetDefaultInstance, NT_GetEntry, NT_Now,
    NT_SetEntryFlags, NT_SetEntryValue, NT_StartServer, NT_Type, NT_Value, NT_ValueData,
    WPI_String,
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

        let mut persist_name: WPI_String = c"networktables.json".into();
        NT_StartServer(inst, &raw mut persist_name, std::ptr::null(), 1735, 5810);

        // Who knows
        sleep(Duration::from_secs(1));

        let mut name: WPI_String = c"/foo".into();
        let foo = NT_GetEntry(inst, &raw mut name);
        let mut val: WPI_String = c"yarr".into();
        let val = NT_Value {
            r#type: NT_Type::NT_STRING,
            last_change: NT_Now(),
            server_time: NT_Now(),
            data: NT_ValueData { v_string: val },
        };
        NT_SetEntryValue(foo, &raw const val);
        NT_SetEntryFlags(foo, NT_EntryFlags::NT_PERSISTENT);
        loop {
            sleep(Duration::from_secs(1));
        }
    }
}
