#![allow(non_camel_case_types, non_snake_case)]

use std::fmt::Debug;

#[repr(C)]
#[derive(Debug, Copy, Clone, Hash)]
pub struct WPI_String {
    str: *const std::ffi::c_char,
    len: usize,
}

pub type NT_Bool = i32;
pub type NT_Handle = u32;
pub type NT_ConnectionDataLogger = NT_Handle;
pub type NT_DataLogger = NT_Handle;
pub type NT_Entry = NT_Handle;
pub type NT_Inst = NT_Handle;
pub type NT_Listener = NT_Handle;
pub type NT_ListenerPoller = NT_Handle;
pub type NT_MultiSubscriber = NT_Handle;
pub type NT_Topic = NT_Handle;
pub type NT_Subscriber = NT_Handle;
pub type NT_Publisher = NT_Handle;

macro_rules! c_enum {
    {$(
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $type:ty {
        $($(#[$memmeta:meta])* $var:ident = $val:expr),+ $(,)?
    })+} => {
        $(
            #[repr(C)]
            #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
            $(#[$meta])*
            $vis struct $name($type);
            impl $name {
                pub fn bits(&self) -> $type {
                    self.0
                }

                $(
                    $(#[$memmeta])*
                    pub const $var: $name = $name($val);
                )+
            }
            impl ::std::ops::BitOr for $name {
                type Output = Self;
                fn bitor(self, rhs: Self) -> Self {
                    $name(self.0 | rhs.0)
                }
            }
            impl ::std::ops::BitOrAssign for $name {
                fn bitor_assign(&mut self, rhs: Self) {
                    self.0 |= rhs.0;
                }
            }
            impl ::std::ops::BitAnd for $name {
                type Output = Self;
                fn bitand(self, rhs: Self) -> Self {
                    $name(self.0 & rhs.0)
                }
            }
            impl ::std::ops::BitAndAssign for $name {
                fn bitand_assign(&mut self, rhs: Self) {
                    self.0 &= rhs.0;
                }
            }
        )+
    };
}

c_enum! {
    /// NetworkTables data types.
    pub enum NT_Type: u32 {
        NT_UNASSIGNED = 0,
        NT_BOOLEAN = 0x01,
        NT_DOUBLE = 0x02,
        NT_STRING = 0x04,
        NT_RAW = 0x08,
        NT_BOOLEAN_ARRAY = 0x10,
        NT_DOUBLE_ARRAY = 0x20,
        NT_STRING_ARRAY = 0x40,
        NT_RPC = 0x80,
        NT_INTEGER = 0x100,
        NT_FLOAT = 0x200,
        NT_INTEGER_ARRAY = 0x400,
        NT_FLOAT_ARRAY = 0x800
    }

    /// NetworkTables entry flags.
    pub enum NT_EntryFlags: u32 {
        NT_PERSISTENT = 0x01,
        NT_RETAINED = 0x02,
        NT_UNCACHED = 0x04
    }

    /// NetworkTables logging levels.
    pub enum NT_LogLevel: u32 {
        NT_LOG_CRITICAL = 50,
        NT_LOG_ERROR = 40,
        NT_LOG_WARNING = 30,
        NT_LOG_INFO = 20,
        NT_LOG_DEBUG = 10,
        NT_LOG_DEBUG1 = 9,
        NT_LOG_DEBUG2 = 8,
        NT_LOG_DEBUG3 = 7,
        NT_LOG_DEBUG4 = 6
    }

    /// Client/server modes
    pub enum NT_NetworkMode: u32 {
        /// Not running
        NT_NET_MODE_NONE = 0x00,
        /// Running in server mode
        NT_NET_MODE_SERVER = 0x01,
        /// Running in NT3 client mode
        NT_NET_MODE_CLIENT3 = 0x02,
        /// Running in NT4 client mode
        NT_NET_MODE_CLIENT4 = 0x04,
        /// Starting (either client or server)
        NT_NET_MODE_STARTING = 0x08,
        /// Running in local-only mode
        NT_NET_MODE_LOCAL = 0x10,
    }

    /// Event notification flags.
    pub enum NT_EventFlags: u32 {
        NT_EVENT_NONE = 0,
        /// Initial listener addition.
        NT_EVENT_IMMEDIATE = 0x01,
        /// Client connected (on server, any client connected).
        NT_EVENT_CONNECTED = 0x02,
        /// Client disconnected (on server, any client disconnected).
        NT_EVENT_DISCONNECTED = 0x04,
        /// Any connection event (connect or disconnect).
        NT_EVENT_CONNECTION = 0x02 |  0x04,
        /// New topic published.
        NT_EVENT_PUBLISH = 0x08,
        /// Topic unpublished.
        NT_EVENT_UNPUBLISH = 0x10,
        /// Topic properties changed.
        NT_EVENT_PROPERTIES = 0x20,
        /// Any topic event (publish, unpublish, or properties changed).
        NT_EVENT_TOPIC = 0x08 | 0x10 | 0x20,
        /// Topic value updated (via network).
        NT_EVENT_VALUE_REMOTE = 0x40,
        /// Topic value updated (local).
        NT_EVENT_VALUE_LOCAL = 0x80,
        /// Topic value updated (network or local).
        NT_EVENT_VALUE_ALL = 0x40 | 0x80,
        /// Log message.
        NT_EVENT_LOGMESSAGE = 0x100,
        /// Time synchronized with server.
        NT_EVENT_TIMESYNC = 0x200,
    }
}

/// Not included in the original header file, but required because of rust union limitations.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NT_ValueDataRaw {
    pub data: *const std::ffi::c_void,
    pub size: usize,
}

/// Not included in the original ntcore header file, but required because of rust union limitations.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NT_ValueDataArray<T: Copy + Clone + Debug> {
    pub arr: *const T,
    pub size: usize,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union NT_ValueData {
    pub v_boolean: NT_Bool,
    pub v_int: i64,
    pub v_float: f32,
    pub v_double: f64,
    pub v_string: WPI_String,
    pub v_raw: NT_ValueDataRaw,
    pub arr_boolean: NT_ValueDataArray<NT_Bool>,
    pub arr_double: NT_ValueDataArray<f64>,
    pub arr_float: NT_ValueDataArray<f32>,
    pub arr_int: NT_ValueDataArray<i64>,
    pub arr_string: NT_ValueDataArray<WPI_String>,
}

/// NetworkTables Entry Value.  Note this is a typed union.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct NT_Value {
    pub r#type: NT_Type,
    pub last_change: i64,
    pub server_time: i64,
    pub data: NT_ValueData,
}

/// NetworkTables Topic Information
#[repr(C)]
#[derive(Copy, Clone, Debug, Hash)]
pub struct NT_TopicInfo {
    /// Topic handle
    pub topic: NT_Topic,
    /// Topic name
    pub name: WPI_String,
    /// Topic type
    pub r#type: NT_Type,
    /// Topic type string
    pub type_str: WPI_String,
    /// Topic properties JSON string
    pub properties: WPI_String,
}

/// NetworkTables Connection Information
#[repr(C)]
#[derive(Copy, Clone, Debug, Hash)]
pub struct NT_ConnectionInfo {
    /**
    /// The remote identifier (as set on the remote node by NT_StartClient4().
     */
    pub remote_id: WPI_String,

    /// The IP address of the remote node.
    pub remote_ip: WPI_String,

    /// The port number of the remote node.
    pub remote_port: u32,

    /**
    /// The last time any update was received from the remote node (same scale as
    /// returned by nt::Now()).
     */
    pub last_update: u64,

    /**
    /// The protocol version being used for this connection.  This in protocol
    /// layer format, so 0x0200 = 2.0, 0x0300 = 3.0).
     */
    pub protocol_version: u32,
}

/// NetworkTables value event data.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct NT_ValueEventData {
    /// Topic handle.
    pub topic: NT_Topic,

    /// Subscriber/entry handle.
    pub subentry: NT_Handle,

    /// The new value.
    pub value: NT_Value,
}

/// NetworkTables log message.
#[repr(C)]
#[derive(Copy, Clone, Debug, Hash)]
pub struct NT_LogMessage {
    /// Log level of the message.  See NT_LogLevel.
    pub level: u32,

    /// The filename of the source file that generated the message.
    pub filename: WPI_String,

    /// The line number in the source file that generated the message.
    pub line: u32,

    /// The message.
    pub message: WPI_String,
}

/// NetworkTables time sync event data.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NT_TimeSyncEventData {
    /// Offset between local time and server time, in microseconds. Add this value
    /// to local time to get the estimated equivalent server time.
    pub serverTimeOffset: i64,

    /// Measured round trip time divided by 2, in microseconds.
    pub rtt2: i64,

    /// If serverTimeOffset and RTT are valid. An event with this set to false is
    /// sent when the client disconnects.
    pub valid: NT_Bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union NT_EventData {
    pub connInfo: NT_ConnectionInfo,
    pub topicInfo: NT_TopicInfo,
    pub valueData: NT_ValueEventData,
    pub logMessage: NT_LogMessage,
    pub timeSyncData: NT_TimeSyncEventData,
}

/// NetworkTables event
#[repr(C)]
#[derive(Copy, Clone)]
pub struct NT_Event {
    /// Listener that triggered this event.
    pub listener: NT_Handle,

    /// Event flags (NT_EventFlags). Also indicates the data included with the
    /// event:
    /// - NT_EVENT_CONNECTED or NT_EVENT_DISCONNECTED: connInfo
    /// - NT_EVENT_PUBLISH, NT_EVENT_UNPUBLISH, or NT_EVENT_PROPERTIES: topicInfo
    /// - NT_EVENT_VALUE_REMOTE, NT_NOTIFY_VALUE_LOCAL: valueData
    /// - NT_EVENT_LOGMESSAGE: logMessage
    /// - NT_EVENT_TIMESYNC: timeSyncData
    pub flags: u32,

    /// Event data; content depends on flags.
    pub data: NT_EventData,
}

/// NetworkTables publish/subscribe options.
pub struct NT_PubSubOptions {
    /// Structure size. Must be set to sizeof(NT_PubSubOptions).
    pub structSize: u32,

    /// Polling storage size for a subscription. Specifies the maximum number of
    /// updates NetworkTables should store between calls to the subscriber's
    /// ReadQueue() function. If zero, defaults to 1 if sendAll is false, 20 if
    /// sendAll is true.
    pub pollStorage: u32,

    /// How frequently changes will be sent over the network, in seconds.
    /// NetworkTables may send more frequently than this (e.g. use a combined
    /// minimum period for all values) or apply a restricted range to this value.
    /// The default is 100 ms.
    pub periodic: f64,

    /// For subscriptions, if non-zero, value updates for ReadQueue() are not
    /// queued for this publisher.
    pub excludePublisher: NT_Publisher,

    /// Send all value changes over the network.
    pub sendAll: NT_Bool,

    /// For subscriptions, don't ask for value changes (only topic announcements).
    pub topicsOnly: NT_Bool,

    /// Perform prefix match on subscriber topic names. Is ignored/overridden by
    /// Subscribe() functions; only present in struct for the purposes of getting
    /// information about subscriptions.
    pub prefixMatch: NT_Bool,

    /// Preserve duplicate value changes (rather than ignoring them).
    pub keepDuplicates: NT_Bool,

    /// For subscriptions, if remote value updates should not be queued for
    /// ReadQueue(). See also disableLocal.
    pub disableRemote: NT_Bool,

    /// For subscriptions, if local value updates should not be queued for
    /// ReadQueue(). See also disableRemote.
    pub disableLocal: NT_Bool,

    /// For entries, don't queue (for ReadQueue) value updates for the entry's
    /// internal publisher.
    pub excludeSelf: NT_Bool,

    /// For subscriptions, don't share the existence of the subscription with the
    /// network. Note this means updates will not be received from the network
    /// unless another subscription overlaps with this one, and the subscription
    /// will not appear in metatopics.
    pub hidden: NT_Bool,
}

extern "C" {}
