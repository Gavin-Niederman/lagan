#![allow(non_camel_case_types, non_snake_case)]

use std::{ffi::CStr, fmt::Debug};
use bitflags::bitflags;

#[repr(C)]
#[derive(Debug, Copy, Clone, Hash)]
pub struct WPI_String {
    pub str: *const std::ffi::c_char,
    pub len: usize,
}

impl From<&CStr> for WPI_String {
    fn from(s: &CStr) -> Self {
        Self {
            str: s.as_ptr() as *const std::ffi::c_char,
            len: s.count_bytes(),
        }
    }
}

pub type WPI_DataLog = std::ffi::c_void;

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

/// Event listener callback function.
///
/// @param data            data pointer provided to callback creation function
/// @param event           event info
pub type NT_ListenerCallback = extern "C" fn(*mut std::ffi::c_void, *const NT_Event);

macro_rules! c_enum {
    {$(
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $type:ty {
        $($(#[$memmeta:meta])* $var:ident = $val:expr),+ $(,)?
    })+} => {
        $(
            #[repr(transparent)]
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
}

bitflags! {
    /// Event notification flags.
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct NT_EventFlags: u32 {
        const NT_EVENT_NONE = 0;
        /// Initial listener addition.
        const NT_EVENT_IMMEDIATE = 0x01;
        /// Client connected (on server, any client connected).
        const NT_EVENT_CONNECTED = 0x02;
        /// Client disconnected (on server, any client disconnected).
        const NT_EVENT_DISCONNECTED = 0x04;
        /// Any connection event (connect or disconnect).
        const NT_EVENT_CONNECTION = 0x02|  0x04;
        /// New topic published.
        const NT_EVENT_PUBLISH = 0x08;
        /// Topic unpublished.
        const NT_EVENT_UNPUBLISH = 0x10;
        /// Topic properties changed.
        const NT_EVENT_PROPERTIES = 0x20;
        /// Any topic event (publish, unpublish, or properties changed).
        const NT_EVENT_TOPIC = 0x08| 0x10 | 0x20;
        /// Topic value updated (via network).
        const NT_EVENT_VALUE_REMOTE = 0x40;
        /// Topic value updated (local).
        const NT_EVENT_VALUE_LOCAL = 0x80;
        /// Topic value updated (network or local).
        const NT_EVENT_VALUE_ALL = 0x40 | 0x80;
        /// Log message.
        const NT_EVENT_LOGMESSAGE = 0x100;
        /// Time synchronized with server.
        const NT_EVENT_TIMESYNC = 0x200;
    }

    /// NetworkTables entry flags.
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct NT_EntryFlags: u32 {
        const NT_PERSISTENT = 0x01;
        const NT_RETAINED = 0x02;
        const NT_UNCACHED = 0x04;
    }
}

/// Not included in the original header file, but required because of rust union limitations.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NT_ValueDataRaw {
    pub data: *const u8,
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
    /// The remote identifier (as set on the remote node by NT_StartClient4().
    pub remote_id: WPI_String,

    /// The IP address of the remote node.
    pub remote_ip: WPI_String,

    /// The port number of the remote node.
    pub remote_port: u32,

    /// The last time any update was received from the remote node (same scale as
    /// returned by nt::Now()).
    pub last_update: u64,

    /// The protocol version being used for this connection.  This in protocol
    /// layer format, so 0x0200 = 2.0, 0x0300 = 3.0).
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
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
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

extern "C" {
    /// Get default instance.
    /// This is the instance used by non-handle-taking functions.
    ///
    /// # Returns
    ///
    /// Instance handle
    pub fn NT_GetDefaultInstance() -> NT_Inst;

    /// Create an instance.
    ///
    /// # Returns
    ///
    /// Instance handle
    pub fn NT_CreateInstance() -> NT_Inst;

    /// Destroy an instance.
    /// The default instance cannot be destroyed.
    ///
    /// # Parameters
    ///
    /// - inst: Instance handle
    pub fn NT_DestroyInstance(inst: NT_Inst);

    /// Get instance handle from another handle.
    pub fn NT_GetInstanceFromHandle(handle: NT_Handle) -> NT_Inst;

    /// Get Entry Handle.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    /// - `name`: Entry name (UTF-8 string).
    ///
    /// # Returns
    ///
    /// Entry handle.
    pub fn NT_GetEntry(inst: NT_Inst, name: *const WPI_String) -> NT_Entry;

    /// Gets the name of the specified entry.
    /// Returns an empty string if the handle is invalid.
    ///
    /// # Parameters
    ///
    /// - `entry`: Entry handle.
    /// - `name`: Entry name (output parameter).
    pub fn NT_GetEntryName(entry: NT_Entry, name: *mut WPI_String);

    /// Gets the type for the specified key, or unassigned if non-existent.
    ///
    /// # Parameters
    ///
    /// - `entry`: Entry handle.
    ///
    /// # Returns
    ///
    /// Entry type.
    pub fn NT_GetEntryType(entry: NT_Entry) -> NT_Type;

    /// Gets the last time the entry was changed.
    /// Returns 0 if the handle is invalid.
    ///
    /// # Parameters
    ///
    /// - entry: entry handle
    ///
    /// # Returns
    ///
    /// Entry last change time
    pub fn NT_GetEntryLastChange(entry: NT_Entry) -> u64;

    /// Get Entry Value.
    ///
    /// Returns copy of current entry value.
    /// Note that one of the type options is "unassigned".
    ///
    /// # Parameters
    ///
    /// - entry: entry handle
    /// - value: storage for returned entry value
    ///
    /// # Note
    ///
    /// It is the caller's responsibility to free value once it's no longer
    /// needed (the utility function NT_DisposeValue() is useful for this
    /// purpose).
    pub fn NT_GetEntryValue(entry: NT_Entry, value: *mut NT_Value);

    /// Get Entry Value.
    ///
    /// Returns copy of current entry value.
    /// Note that one of the type options is "unassigned".
    ///
    /// # Parameters
    ///
    /// - entry: entry handle
    /// - types: bitmask of NT_Type values; 0 is treated specially
    ///   as a "don't care"
    /// - value: storage for returned entry value
    ///
    /// # Note
    ///
    /// It is the caller's responsibility to free value once it's no longer
    /// needed (the utility function NT_DisposeValue() is useful for this
    /// purpose).
    pub fn NT_GetEntryValueType(entry: NT_Entry, types: NT_Type, value: *mut NT_Value);

    /// Set Default Entry Value.
    ///
    /// Returns copy of current entry value if it exists.
    /// Otherwise, sets passed in value, and returns set value.
    /// Note that one of the type options is "unassigned".
    ///
    /// # Parameters
    ///
    /// - entry: entry handle
    /// - default_value: value to be set if name does not exist
    ///
    /// # Returns
    ///
    /// 0 on error (value not set), 1 on success
    pub fn NT_SetDefaultEntryValue(entry: NT_Entry, default_value: *const NT_Value) -> NT_Bool;

    /// Set Entry Value.
    ///
    /// Sets new entry value.  If type of new value differs from the type of the
    /// currently stored entry, returns error and does not update value.
    ///
    /// # Parameters
    ///
    /// - entry: entry handle
    /// - value: new entry value
    ///
    /// # Returns
    ///
    /// 0 on error (type mismatch), 1 on success
    pub fn NT_SetEntryValue(entry: NT_Entry, value: *const NT_Value) -> NT_Bool;

    /// Set Entry Flags.
    ///
    /// # Parameters
    ///
    /// - entry: entry handle
    /// - flags: flags value (bitmask of NT_EntryFlags)
    pub fn NT_SetEntryFlags(entry: NT_Entry, flags: NT_EntryFlags);

    /// Get Entry Flags.
    ///
    /// # Parameters
    ///
    /// - entry: entry handle
    ///
    /// # Returns
    ///
    /// Flags value (bitmask of NT_EntryFlags)
    pub fn NT_GetEntryFlags(entry: NT_Entry) -> NT_EntryFlags;

    /// Read Entry Queue.
    ///
    /// Returns new entry values since last call. The returned array must be freed
    /// using NT_DisposeValueArray().
    ///
    /// # Parameters
    ///
    /// - subentry: subscriber or entry handle
    /// - count: count of items in returned array (output)
    ///
    /// # Returns
    ///
    /// entry value array; returns NULL and count=0 if no new values
    pub fn NT_ReadQueueValue(subentry: NT_Handle, count: *mut isize) -> *mut NT_Value;

    /// Read Entry Queue.
    ///
    /// Returns new entry values since last call. The returned array must be freed
    /// using NT_DisposeValueArray().
    ///
    /// # Parameters
    ///
    /// - subentry: subscriber or entry handle
    /// - types: bitmask of NT_Type values; 0 is treated specially
    ///   as a "don't care"
    /// - count: count of items in returned array (output)
    ///
    /// # Returns
    ///
    /// entry value array; returns NULL and count=0 if no new values
    pub fn NT_ReadQueueValueType(
        subentry: NT_Handle,
        types: u32,
        count: *mut isize,
    ) -> *mut NT_Value;

    /// Get Published Topic Handles.
    ///
    /// Returns an array of topic handles.  The results are optionally
    /// filtered by string prefix and type to only return a subset of all
    /// topics.
    ///
    /// # Parameters
    ///
    /// - inst: instance handle
    /// - prefix: name required prefix; only topics whose name
    ///   starts with this string are returned
    /// - types: bitmask of NT_Type values; 0 is treated specially
    ///   as a "don't care"
    /// - count: output parameter; set to length of returned array
    ///
    /// # Returns
    ///
    /// Array of topic handles.
    pub fn NT_GetTopics(
        inst: NT_Inst,
        prefix: *const WPI_String,
        types: u32,
        count: *mut isize,
    ) -> *mut NT_Topic;

    /// Get Published Topic Handles.
    ///
    /// Returns an array of topic handles.  The results are optionally
    /// filtered by string prefix and type to only return a subset of all
    /// topics.
    ///
    /// # Parameters
    ///
    /// - inst: instance handle
    /// - prefix: name required prefix; only topics whose name
    ///   starts with this string are returned
    /// - types: array of type strings
    /// - types_len: number of elements in types array
    /// - count: output parameter; set to length of returned array
    ///
    /// # Returns
    ///
    /// Array of topic handles.
    pub fn NT_GetTopicsStr(
        inst: NT_Inst,
        prefix: *const WPI_String,
        types: *const WPI_String,
        types_len: isize,
        count: *mut isize,
    ) -> *mut NT_Topic;

    /// Get Topics.
    ///
    /// Returns an array of topic information (handle, name, type).  The results are
    /// optionally filtered by string prefix and type to only return a subset
    /// of all topics.
    ///
    /// # Parameters
    ///
    /// - inst: instance handle
    /// - prefix: name required prefix; only topics whose name
    ///   starts with this string are returned
    /// - types: bitmask of NT_Type values; 0 is treated specially
    ///   as a "don't care"
    /// - count: output parameter; set to length of returned array
    ///
    /// # Returns
    ///
    /// Array of topic information.
    pub fn NT_GetTopicInfos(
        inst: NT_Inst,
        prefix: *const WPI_String,
        types: u32,
        count: *mut usize,
    ) -> *mut NT_TopicInfo;

    /// Get Topics.
    ///
    /// Returns an array of topic information (handle, name, type).  The results are
    /// optionally filtered by string prefix and type to only return a subset
    /// of all topics.
    ///
    /// # Parameters
    ///
    /// - inst: instance handle
    /// - prefix: name required prefix; only topics whose name
    ///   starts with this string are returned
    /// - types: array of type strings
    /// - types_len: number of elements in types array
    /// - count: output parameter; set to length of returned array
    ///
    /// # Returns
    ///
    /// Array of topic information.
    pub fn NT_GetTopicInfosStr(
        inst: NT_Inst,
        prefix: *const WPI_String,
        types: *const WPI_String,
        types_len: isize,
        count: *mut isize,
    ) -> *mut NT_TopicInfo;

    /// Gets Topic Information.
    ///
    /// Returns information about a topic (name and type).
    ///
    /// - topic: handle
    /// - info: information (output)
    ///
    /// # Returns
    ///
    /// True if successful, false on error.
    pub fn NT_GetTopicInfo(topic: NT_Topic, info: *mut NT_TopicInfo) -> NT_Bool;

    /// Gets Topic Handle.
    ///
    /// Returns topic handle.
    ///
    /// # Parameters
    ///
    /// - inst: instance handle
    /// - name: topic name
    ///
    /// # Returns
    ///
    /// Topic handle.
    pub fn NT_GetTopic(inst: NT_Inst, name: *const WPI_String) -> NT_Topic;

    /// Gets the name of the specified topic.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - name: topic name (output); return length of 0 and nullptr if
    ///   handle is invalid.
    pub fn NT_GetTopicName(topic: NT_Topic, name: *mut WPI_String);

    /// Gets the type for the specified topic, or unassigned if non existent.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    ///
    /// # Returns
    ///
    /// Topic type
    pub fn NT_GetTopicType(topic: NT_Topic) -> NT_Type;

    /// Gets the type string for the specified topic.  This may have more information
    /// than the numeric type (especially for raw values).
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - type: topic type string (output)
    pub fn NT_GetTopicTypeString(topic: NT_Topic, r#type: *mut WPI_String);

    /// Sets the persistent property of a topic.  If true, the stored value is
    /// persistent through server restarts.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - value: True for persistent, false for not persistent.
    pub fn NT_SetTopicPersistent(topic: NT_Topic, value: NT_Bool);

    /// Gets the persistent property of a topic.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    ///
    /// # Returns
    ///
    /// persistent property value
    pub fn NT_GetTopicPersistent(topic: NT_Topic) -> NT_Bool;

    /// Sets the retained property of a topic.  If true, the server retains the
    /// topic even when there are no publishers.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - value: new retained property value
    pub fn NT_SetTopicRetained(topic: NT_Topic, value: NT_Bool);

    /// Gets the retained property of a topic.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    ///
    /// # Returns
    ///
    /// retained property value
    pub fn NT_GetTopicRetained(topic: NT_Topic) -> NT_Bool;

    /// Sets the cached property of a topic.  If true, the server and clients will
    /// store the latest value, allowing the value to be read (and not just accessed
    /// through event queues and listeners).
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - value: True for cached, false for not cached
    pub fn NT_SetTopicCached(topic: NT_Topic, value: NT_Bool);

    /// Gets the cached property of a topic.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    ///
    /// # Return
    ///
    /// cached property value
    pub fn NT_GetTopicCached(topic: NT_Topic) -> NT_Bool;

    /// Determine if topic exists (e.g. has at least one publisher).
    ///
    /// # Parameters
    ///
    /// - handle: Topic, entry, or subscriber handle.
    ///
    /// # Returns
    ///
    /// True if topic exists.
    pub fn NT_GetTopicExists(handle: NT_Handle) -> NT_Bool;

    /// Gets the current value of a property (as a JSON string).
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - name: property name
    /// - property: JSON string (output)
    pub fn NT_GetTopicProperty(topic: NT_Topic, name: *const WPI_String, property: *mut WPI_String);

    /// Sets a property value.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - name: property name
    /// - value: property value (JSON string)
    pub fn NT_SetTopicProperty(
        topic: NT_Topic,
        name: *const WPI_String,
        value: *const WPI_String,
    ) -> NT_Bool;

    /// Deletes a property.  Has no effect if the property does not exist.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - name: property name
    pub fn NT_DeleteTopicProperty(topic: NT_Topic, name: *const WPI_String);

    /// Gets all topic properties as a JSON string.  Each key in the object
    /// is the property name, and the corresponding value is the property value.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - properties: JSON string (output)
    pub fn NT_GetTopicProperties(topic: NT_Topic, properties: *mut WPI_String);

    /// Updates multiple topic properties.  Each key in the passed-in JSON object is
    /// the name of the property to add/update, and the corresponding value is the
    /// property value to set for that property.  Null values result in deletion
    /// of the corresponding property.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - properties: JSON object string with keys to add/update/delete
    ///
    /// # Returns
    ///
    /// False if properties are not a valid JSON object
    pub fn NT_SetTopicProperties(topic: NT_Topic, properties: *const WPI_String) -> NT_Bool;

    /// Creates a new subscriber to value changes on a topic.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - type: expected type
    /// - typeStr: expected type string
    /// - options: subscription options
    ///
    /// # Returns
    ///
    /// Subscriber handle
    pub fn NT_Subscribe(
        topic: NT_Topic,
        r#type: NT_Type,
        typeStr: *const WPI_String,
        options: *const NT_PubSubOptions,
    ) -> NT_Subscriber;

    /// Stops subscriber.
    ///
    /// # Returns
    ///
    /// sub subscriber handle
    pub fn NT_Unsubscribe(sub: NT_Subscriber);

    /// Creates a new publisher to a topic.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - type: type
    /// - typeStr: type string
    /// - options: publish options
    ///
    /// # Returns
    ///
    /// Publisher handle
    pub fn NT_Publish(
        topic: NT_Topic,
        r#type: NT_Type,
        typeStr: *const WPI_String,
        options: *const NT_PubSubOptions,
    ) -> NT_Publisher;

    /// Creates a new publisher to a topic.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - type: type
    /// - typeStr: type string
    /// - properties: initial properties (JSON object)
    /// - options: publish options
    ///
    /// # Returns
    ///
    /// Publisher handle
    pub fn NT_PublishEx(
        topic: NT_Topic,
        r#type: NT_Type,
        typeStr: *const WPI_String,
        properties: *const WPI_String,
        options: *const NT_PubSubOptions,
    ) -> NT_Publisher;

    /// Stops publisher.
    ///
    /// # Parameters
    ///
    /// pubentry publisher/entry handle
    pub fn NT_Unpublish(pubentry: NT_Handle);

    /// Creates a new entry (subscriber and weak publisher) to a topic.
    ///
    /// # Parameters
    ///
    /// - topic: topic handle
    /// - type: type
    /// - typeStr: type string
    /// - options: publish options
    ///
    /// # Returns
    ///
    /// Entry handle
    pub fn NT_GetEntryEx(
        topic: NT_Topic,
        r#type: NT_Type,
        typeStr: *const WPI_String,
        options: *const NT_PubSubOptions,
    ) -> NT_Entry;

    /// Stops entry subscriber/publisher.
    ///
    /// # Parameters
    ///
    /// - entry: entry handle
    pub fn NT_ReleaseEntry(entry: NT_Entry);

    /// Stops entry/subscriber/publisher.
    ///
    /// # Parameters
    ///
    /// - pubsubentry: entry/subscriber/publisher handle
    pub fn NT_Release(pubsubentry: NT_Handle);

    /// Gets the topic handle from an entry/subscriber/publisher handle.
    ///
    /// # Parameters
    ///
    /// - pubsubentry: entry/subscriber/publisher handle
    ///
    /// # Returns
    ///
    /// Topic handle
    pub fn NT_GetTopicFromHandle(pubsubentry: NT_Handle) -> NT_Topic;

    /// Subscribes to multiple topics based on one or more topic name prefixes. Can
    /// be used in combination with a Value Listener or ReadQueueValue() to get value
    /// changes across all matching topics.
    ///
    /// # Parameters
    ///
    /// - inst: instance handle
    /// - prefixes: topic name prefixes
    /// - prefixes_len: number of elements in prefixes array
    /// - options: subscriber options
    ///
    /// # Returns
    ///
    /// subscriber handle
    pub fn NT_SubscribeMultiple(
        inst: NT_Inst,
        prefixes: *const WPI_String,
        prefixes_len: usize,
        options: *const NT_PubSubOptions,
    ) -> NT_MultiSubscriber;

    /// Unsubscribes a multi-subscriber.
    ///
    /// # Parameters
    ///
    /// sub multi-subscriber handle
    pub fn NT_UnsubscribeMultiple(sub: NT_MultiSubscriber);

    /// Creates a listener poller.
    ///
    /// A poller provides a single queue of poll events.  Events linked to this
    /// poller (using NT_AddPolledXListener()) will be stored in the queue and
    /// must be collected by calling NT_ReadListenerQueue().
    /// The returned handle must be destroyed with NT_DestroyListenerPoller().
    ///
    /// # Parameters
    ///
    /// - inst: instance handle
    ///
    /// # Returns
    ///
    /// poller handle
    pub fn NT_CreateListenerPoller(inst: NT_Inst) -> NT_ListenerPoller;

    /// Destroys a listener poller.  This will abort any blocked polling
    /// call and prevent additional events from being generated for this poller.
    ///
    /// # Parameters
    ///
    /// - poller: poller handle
    pub fn NT_DestroyListenerPoller(poller: NT_ListenerPoller);

    /// Read notifications.
    ///
    /// # Parameters
    ///
    /// - poller: poller handle
    /// - len: length of returned array (output)
    ///
    /// # Returns
    ///
    /// Array of events.  Returns NULL and len=0 if no events since last call.
    pub fn NT_ReadListenerQueue(poller: NT_ListenerPoller, len: *mut usize) -> *mut NT_Event;

    /// Removes a listener.
    ///
    /// # Parameters
    ///
    /// - listener: Listener handle to remove
    pub fn NT_RemoveListener(listener: NT_Listener);

    /// Wait for the listener queue to be empty. This is primarily useful
    /// for deterministic testing. This blocks until either the listener
    /// queue is empty (e.g. there are no more events that need to be passed along to
    /// callbacks or poll queues) or the timeout expires.
    ///
    /// # Parameters
    ///
    /// - handle:  handle
    /// - timeout: timeout, in seconds. Set to 0 for non-blocking behavior, or a
    ///   negative value to block indefinitely
    ///
    /// # Returns
    ///
    /// False if timed out, otherwise true.
    pub fn NT_WaitForListenerQueue(handle: NT_Handle, timeout: f64) -> NT_Bool;

    /// Create a listener for changes to topics with names that start with
    /// the given prefix. This creates a corresponding internal subscriber with the
    /// lifetime of the listener.
    ///
    /// # Parameters
    ///
    /// - inst: Instance handle
    /// - prefix: Topic name string prefix
    /// - mask: Bitmask of NT_EventFlags values (only topic and value events will
    ///   be generated)
    /// - data: Data passed to callback function
    /// - callback: Listener function
    ///
    /// # Returns
    ///
    /// Listener handle
    pub fn NT_AddListenerSingle(
        inst: NT_Inst,
        prefix: *const WPI_String,
        mask: u32,
        data: *mut std::ffi::c_void,
        callback: NT_ListenerCallback,
    ) -> NT_Listener;

    /// Create a listener for changes to topics with names that start with any of
    /// the given prefixes. This creates a corresponding internal subscriber with the
    /// lifetime of the listener.
    ///
    /// # Parameters
    ///
    /// - inst: Instance handle
    /// - prefixes: Topic name string prefixes
    /// - prefixes_len: Number of elements in prefixes array
    /// - mask: Bitmask of NT_EventFlags values (only topic and value events will
    ///   be generated)
    /// - data: Data passed to callback function
    /// - callback: Listener function
    ///
    /// # Returns
    ///
    /// Listener handle
    pub fn NT_AddListenerMultiple(
        inst: NT_Inst,
        prefixes: *const WPI_String,
        prefixes_len: usize,
        mask: u32,
        data: *mut std::ffi::c_void,
        callback: NT_ListenerCallback,
    ) -> NT_Listener;

    /// Create a listener.
    ///
    /// Some combinations of handle and mask have no effect:
    /// - connection and log message events are only generated on instances
    /// - topic and value events are only generated on non-instances
    ///
    /// Adding value and topic events on a topic will create a corresponding internal
    /// subscriber with the lifetime of the listener.
    ///
    /// Adding a log message listener through this function will only result in
    /// events at NT_LOG_INFO or higher; for more customized settings, use
    /// NT_AddLogger().
    ///
    /// # Parameters
    ///
    /// - handle: Handle
    /// - mask: Bitmask of NT_EventFlags values
    /// - data: Data passed to callback function
    /// - callback: Listener function
    ///
    /// # Returns
    ///
    /// Listener handle
    pub fn NT_AddListener(
        handle: NT_Handle,
        mask: u32,
        data: *mut std::ffi::c_void,
        callback: NT_ListenerCallback,
    ) -> NT_Listener;

    /// Creates a polled topic listener. This creates a corresponding internal
    /// subscriber with the lifetime of the listener.
    /// The caller is responsible for calling NT_ReadListenerQueue() to poll.
    ///
    /// # Parameters
    ///
    /// - poller: poller handle
    /// - prefix: UTF-8 string prefix
    /// - mask: NT_EventFlags bitmask (only topic and value events
    ///   will be generated)
    ///
    /// # Returns
    ///
    /// Listener handle
    pub fn NT_AddPolledListenerSingle(
        poller: NT_ListenerPoller,
        prefix: *const WPI_String,
        mask: u32,
    ) -> NT_Listener;

    /// Creates a polled topic listener. This creates a corresponding internal
    /// subscriber with the lifetime of the listener.
    /// The caller is responsible for calling NT_ReadListenerQueue() to poll.
    ///
    /// # Parameters
    ///
    /// - `poller`: Poller handle.
    /// - `prefixes`: Array of UTF-8 string prefixes.
    /// - `prefixes_len`: Length of prefixes array.
    /// - `mask`: NT_EventFlags bitmask (only topic and value events will be generated).
    ///
    /// # Returns
    ///
    /// Listener handle.
    pub fn NT_AddPolledListenerMultiple(
        poller: NT_ListenerPoller,
        prefixes: *const WPI_String,
        prefixes_len: usize,
        mask: u32,
    ) -> NT_Listener;

    /// Creates a polled listener.
    /// The caller is responsible for calling NT_ReadListenerQueue() to poll.
    ///
    /// Some combinations of handle and mask have no effect:
    ///
    /// - connection and log message events are only generated on instances
    /// - topic and value events are only generated on non-instances
    ///
    /// Adding value and topic events on a topic will create a corresponding internal
    /// subscriber with the lifetime of the listener.
    ///
    /// Adding a log message listener through this function will only result in
    /// events at NT_LOG_INFO or higher; for more customized settings, use
    /// NT_AddPolledLogger.
    ///
    /// # Parameters
    ///
    /// - `poller`: Poller handle.
    /// - `handle`: Handle.
    /// - `mask`: NT_NotifyKind bitmask.
    ///
    /// # Returns
    ///
    /// Listener handle.
    pub fn NT_AddPolledListener(
        poller: NT_ListenerPoller,
        handle: NT_Handle,
        mask: u32,
    ) -> NT_Listener;

    /// Starts local-only operation. Prevents calls to NT_StartServer or
    /// NT_StartClient from taking effect. Has no effect if NT_StartServer or
    /// NT_StartClient has already been called.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    pub fn NT_StartLocal(inst: NT_Inst);

    /// Stops local-only operation. NT_StartServer or NT_StartClient can be called
    /// after this call to start a server or client.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    pub fn NT_StopLocal(inst: NT_Inst);

    /// Starts a server using the specified filename, listening address, and port.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    /// - `persist_filename`: The name of the persist file to use (UTF-8 string, null-terminated).
    /// - `listen_address`: The address to listen on, or null to listen on any address (UTF-8 string, null-terminated).
    /// - `port3`: Port to communicate over (NT3).
    /// - `port4`: Port to communicate over (NT4).
    pub fn NT_StartServer(
        inst: NT_Inst,
        persist_filename: *const WPI_String,
        listen_address: *const WPI_String,
        port3: u32,
        port4: u32,
    );

    /// Stops the server if it is running.
    ///
    /// # Parameters
    ///
    /// inst: instance handle
    pub fn NT_StopServer(inst: NT_Inst);

    /// Starts a NT3 client. Use NT_SetServer or NT_SetServerTeam to set the server
    /// name and port.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    /// - `identity`: Network identity to advertise (cannot be empty string).
    pub fn NT_StartClient3(inst: NT_Inst, identity: *const WPI_String);

    /// Starts a NT4 client. Use NT_SetServer or NT_SetServerTeam to set the server
    /// name and port.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    /// - `identity`: Network identity to advertise (cannot be empty string).
    pub fn NT_StartClient4(inst: NT_Inst, identity: *const WPI_String);

    /// Stops the client if it is running.
    ///
    /// # Parameters
    ///
    /// - inst: instance handle
    pub fn NT_StopClient(inst: NT_Inst);

    /// Sets server address and port for client (without restarting client).
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    /// - `server_name`: Server name (UTF-8 string, null-terminated).
    /// - `port`: Port to communicate over.
    pub fn NT_SetServer(inst: NT_Inst, server_name: *const WPI_String, port: u32);

    /// Sets server addresses for client (without restarting client).
    /// The client will attempt to connect to each server in round-robin fashion.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    /// - `count`: Length of the `server_names` and `ports` arrays.
    /// - `server_names`: Array of server names (each a UTF-8 string, null-terminated).
    /// - `ports`: Array of ports to communicate over (one for each server).
    pub fn NT_SetServerMulti(
        inst: NT_Inst,
        count: usize,
        server_names: *const WPI_String,
        ports: *const u32,
    );

    /// Sets server addresses and port for client (without restarting client).
    /// Connects using commonly known robot addresses for the specified team.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    /// - `team`: Team number.
    /// - `port`: Port to communicate over.
    pub fn NT_SetServerTeam(inst: NT_Inst, team: u32, port: u32);

    /// Disconnects the client if it's running and connected. This will automatically
    /// start reconnection attempts to the current server list.
    ///
    /// # Parameters
    ///
    /// - inst: instance handle
    pub fn NT_Disconnect(inst: NT_Inst);

    /// Starts requesting server address from Driver Station.
    /// This connects to the Driver Station running on localhost to obtain the
    /// server IP address.
    ///
    /// # Parameters
    /// - `inst`: Instance handle.
    /// - `port`: Server port to use in combination with IP from DS.
    pub fn NT_StartDSClient(inst: NT_Inst, port: u32);

    /// Stops requesting server address from Driver Station.
    ///
    /// # Parameters
    ///
    /// - inst: instance handle
    pub fn NT_StopDSClient(inst: NT_Inst);

    /// Flush local updates.
    ///
    /// Forces an immediate flush of all local changes to the client/server.
    /// This does not flush to the network.
    ///
    /// Normally this is done on a regularly scheduled interval.
    ///
    /// # Parameters
    ///
    /// - inst: instance handle
    pub fn NT_FlushLocal(inst: NT_Inst);

    /// Flush to network.
    ///
    /// Forces an immediate flush of all local entry changes to network.
    /// Normally this is done on a regularly scheduled interval (set
    /// by update rates on individual publishers).
    ///
    /// Note: flushes are rate limited to avoid excessive network traffic.  If
    /// the time between calls is too short, the flush will occur after the minimum
    /// time elapses (rather than immediately).
    ///
    /// # Parameters
    ///
    /// - inst: instance handle
    pub fn NT_Flush(inst: NT_Inst);

    /// Get information on the currently established network connections.
    /// If operating as a client, this will return either zero or one values.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    /// - `count`: Returns the number of elements in the array.
    ///
    /// # Returns
    ///
    /// Array of connection information.
    ///
    /// It is the caller's responsibility to free the array. The
    /// `NT_DisposeConnectionInfoArray` function is useful for this purpose.
    pub fn NT_GetConnections(inst: NT_Inst, count: *mut usize) -> *mut NT_ConnectionInfo;

    /**
     * Return whether or not the instance is connected to another node.
     *
     * @param inst  instance handle
     * @return True if connected.
     */
    pub fn NT_IsConnected(inst: NT_Inst) -> NT_Bool;

    /// Get the time offset between server time and local time. Add this value to
    /// local time to get the estimated equivalent server time. In server mode, this
    /// always returns a valid value of 0. In client mode, this returns the time
    /// offset only if the client and server are connected and have exchanged
    /// synchronization messages. Note the time offset may change over time as it is
    /// periodically updated; to receive updates as events, add a listener to the
    /// "time sync" event.
    ///
    /// # Parameters
    /// - `inst`: Instance handle.
    /// - `valid`: Set to true if the return value is valid, false otherwise (output).
    ///
    /// # Returns
    /// Time offset in microseconds (if valid is set to true).
    pub fn NT_GetServerTimeOffset(inst: NT_Inst, valid: *mut NT_Bool) -> i64;

    /// Frees value memory.
    ///
    /// # Parameters
    /// - `value`: Value to free.
    pub fn NT_DisposeValue(value: *mut NT_Value);

    /// Initializes an NT_Value.
    /// Sets type to NT_UNASSIGNED and clears the rest of the struct.
    ///
    /// # Parameters
    /// - `value`: Value to initialize.
    pub fn NT_InitValue(value: *mut NT_Value);

    /// Frees an array of NT_Values.
    ///
    /// # Parameters
    /// - `arr`: Pointer to the value array to free.
    /// - `count`: Number of elements in the array.
    ///
    /// Note that the individual NT_Values in the array should NOT be
    /// freed before calling this. This function will free all the values
    /// individually.
    pub fn NT_DisposeValueArray(arr: *mut NT_Value, count: usize);

    /// Disposes a connection info array.
    ///
    /// # Parameters
    /// - `arr`: Pointer to the array to dispose.
    /// - `count`: Number of elements in the array.
    pub fn NT_DisposeConnectionInfoArray(arr: *mut NT_ConnectionInfo, count: usize);

    /// Disposes a topic info array.
    ///
    /// # Parameters
    /// - `arr`: Pointer to the array to dispose.
    /// - `count`: Number of elements in the array.
    pub fn NT_DisposeTopicInfoArray(arr: *mut NT_TopicInfo, count: usize);

    /// Disposes a single topic info (as returned by NT_GetTopicInfo).
    ///
    /// # Parameters
    /// - `info`: Pointer to the info to dispose.
    pub fn NT_DisposeTopicInfo(info: *mut NT_TopicInfo);

    /// Disposes an event array.
    ///
    /// # Parameters
    /// - `arr`: Pointer to the array to dispose.
    /// - `count`: Number of elements in the array.
    pub fn NT_DisposeEventArray(arr: *mut NT_Event, count: usize);

    /// Disposes a single event.
    ///
    /// # Parameters
    /// - `event`: Pointer to the event to dispose.
    pub fn NT_DisposeEvent(event: *mut NT_Event);

    /// Returns monotonic current time in 1 us increments.
    /// This is the same time base used for entry and connection timestamps.
    /// This function by default simply wraps WPI_Now(), but if NT_SetNow() is
    /// called, this function instead returns the value passed to NT_SetNow();
    /// this can be used to reduce overhead.
    ///
    /// # Returns
    ///
    /// Timestamp
    pub fn NT_Now() -> i64;

    /// Sets the current timestamp used for timestamping values that do not
    /// provide a timestamp (e.g. a value of 0 is passed).  For consistency,
    /// it also results in NT_Now() returning the set value.  This should generally
    /// be used only if the overhead of calling WPI_Now() is a concern.
    /// If used, it should be called periodically with the value of WPI_Now().
    ///
    /// # Parameters
    ///
    /// - timestamp: timestamp (1 us increments)
    pub fn NT_SetNow(timestamp: i64);

    /// Starts logging entry changes to a DataLog.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    /// - `log`: Data log object; lifetime must extend until `StopEntryDataLog` is
    ///         called or the instance is destroyed.
    /// - `prefix`: Only store entries with names that start with this prefix;
    ///            the prefix is not included in the data log entry name.
    /// - `logPrefix`: Prefix to add to data log entry names.
    ///
    /// # Returns
    ///
    /// Data logger handle.
    pub fn NT_StartEntryDataLog(
        inst: NT_Inst,
        log: *mut WPI_DataLog,
        prefix: *const WPI_String,
        logPrefix: *const WPI_String,
    ) -> NT_DataLogger;

    /// Stops logging entry changes to a DataLog.
    ///
    /// # Parameters
    ///
    /// - logger: data logger handle
    pub fn NT_StopEntryDataLog(logger: NT_DataLogger);

    /// Starts logging connection changes to a DataLog.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    /// - `log`: Data log object; lifetime must extend until `StopConnectionDataLog`
    ///         is called or the instance is destroyed.
    /// - `name`: Data log entry name.
    ///
    /// # Returns
    ///
    /// Data logger handle.
    pub fn NT_StartConnectionDataLog(
        inst: NT_Inst,
        log: *mut WPI_DataLog,
        name: *const WPI_String,
    ) -> NT_ConnectionDataLogger;

    /// Stops logging connection changes to a DataLog.
    ///
    /// # Parameters
    ///
    /// - logger: data logger handle
    pub fn NT_StopConnectionDataLog(logger: NT_ConnectionDataLogger);

    /// Add logger callback function. By default, log messages are sent to stderr;
    /// this function sends log messages to the provided callback function instead.
    /// The callback function will only be called for log messages with level
    /// greater than or equal to `min_level` and less than or equal to `max_level`;
    /// messages outside this range will be silently ignored.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    /// - `min_level`: Minimum log level.
    /// - `max_level`: Maximum log level.
    /// - `data`: Data pointer to pass to `func`.
    /// - `func`: Listener callback function.
    ///
    /// # Returns
    ///
    /// Listener handle.
    pub fn NT_AddLogger(
        inst: NT_Inst,
        min_level: u32,
        max_level: u32,
        data: *mut std::ffi::c_void,
        func: NT_ListenerCallback,
    ) -> NT_Listener;

    /// Set the log level for a listener poller. Events will only be generated for
    /// log messages with level greater than or equal to `min_level` and less than or
    /// equal to `max_level`; messages outside this range will be silently ignored.
    ///
    /// # Parameters
    ///
    /// - `poller`: Poller handle.
    /// - `min_level`: Minimum log level.
    /// - `max_level`: Maximum log level.
    ///
    /// # Returns
    ///
    /// Listener handle.
    pub fn NT_AddPolledLogger(
        poller: NT_ListenerPoller,
        min_level: u32,
        max_level: u32,
    ) -> NT_Listener;

    /// Returns whether there is a data schema already registered with the given
    /// name. This does NOT perform a check as to whether the schema has already
    /// been published by another node on the network.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    /// - `name`: Name (the string passed as the data type for topics using this schema).
    ///
    /// # Returns
    ///
    /// True if schema already registered.
    pub fn NT_HasSchema(inst: NT_Inst, name: *const WPI_String) -> NT_Bool;

    /// Registers a data schema. Data schemas provide information for how a
    /// certain data type string can be decoded. The type string of a data schema
    /// indicates the type of the schema itself (e.g. "protobuf" for protobuf
    /// schemas, "struct" for struct schemas, etc). In NetworkTables, schemas are
    /// published just like normal topics, with the name being generated from the
    /// provided name: "/.schema/<name>". Duplicate calls to this function with
    /// the same name are silently ignored.
    ///
    /// # Parameters
    ///
    /// - `inst`: Instance handle.
    /// - `name`: Name (the string passed as the data type for topics using this schema).
    /// - `type`: Type of schema (e.g. "protobuf", "struct", etc).
    /// - `schema`: Schema data.
    /// - `schema_size`: Size of schema data.
    pub fn NT_AddSchema(
        inst: NT_Inst,
        name: *const WPI_String,
        type_: *const WPI_String,
        schema: *const u8,
        schema_size: usize,
    );

    /// Allocates an array of chars.
    /// Note that the size is the number of elements, and not the
    /// specific number of bytes to allocate. That is calculated internally.
    ///
    /// # Parameters
    /// - `size`: The number of elements the array will contain.
    ///
    /// # Returns
    /// The allocated char array.
    ///
    /// After use, the array should be freed using the `NT_FreeCharArray()` function.
    pub fn NT_AllocateCharArray(size: usize) -> *mut std::ffi::c_char;

    /// Allocates an array of booleans.
    /// Note that the size is the number of elements, and not the
    /// specific number of bytes to allocate. That is calculated internally.
    ///
    /// # Parameters
    /// - `size`: The number of elements the array will contain.
    ///
    /// # Returns
    /// The allocated boolean array.
    ///
    /// After use, the array should be freed using the `NT_FreeBooleanArray()` function.
    pub fn NT_AllocateBooleanArray(size: usize) -> *mut bool;

    /// Allocates an array of integers.
    /// Note that the size is the number of elements, and not the
    /// specific number of bytes to allocate. That is calculated internally.
    ///
    /// # Parameters
    /// - `size`: The number of elements the array will contain.
    ///
    /// # Returns
    /// The allocated integer array.
    ///
    /// After use, the array should be freed using the `NT_FreeIntegerArray()` function.
    pub fn NT_AllocateIntegerArray(size: usize) -> *mut i64;

    /// Allocates an array of floats.
    /// Note that the size is the number of elements, and not the
    /// specific number of bytes to allocate. That is calculated internally.
    ///
    /// # Parameters
    /// - `size`: The number of elements the array will contain.
    ///
    /// # Returns
    /// The allocated float array.
    ///
    /// After use, the array should be freed using the `NT_FreeFloatArray()` function.
    pub fn NT_AllocateFloatArray(size: usize) -> *mut f32;

    /// Allocates an array of doubles.
    /// Note that the size is the number of elements, and not the
    /// specific number of bytes to allocate. That is calculated internally.
    ///
    /// # Parameters
    ///
    /// - `size`: The number of elements the array will contain.
    ///
    /// # Returns
    ///
    /// The allocated double array.
    ///
    /// After use, the array should be freed using the `NT_FreeDoubleArray()` function.
    pub fn NT_AllocateDoubleArray(size: usize) -> *mut f64;

    /// Frees an array of chars.
    ///
    /// # Parameters
    ///
    /// - `v_char`: Pointer to the char array to free.
    pub fn NT_FreeCharArray(v_char: *mut std::ffi::c_char);

    /// Frees an array of booleans.
    ///
    /// # Parameters
    ///
    /// - `v_boolean`: Pointer to the boolean array to free.
    pub fn NT_FreeBooleanArray(v_boolean: *mut bool);

    /// Frees an array of integers.
    ///
    /// # Parameters
    ///
    /// - `v_int`: Pointer to the integer array to free.
    pub fn NT_FreeIntegerArray(v_int: *mut i64);

    /// Frees an array of floats.
    ///
    /// # Parameters
    ///
    /// - `v_float`: Pointer to the float array to free.
    pub fn NT_FreeFloatArray(v_float: *mut f32);

    /// Frees an array of doubles.
    ///
    /// # Parameters
    ///
    /// - `v_double`: Pointer to the double array to free.
    pub fn NT_FreeDoubleArray(v_double: *mut f64);

    /// Returns the type of an NT_Value struct.
    /// Note that one of the type options is "unassigned".
    ///
    /// # Parameters
    ///
    /// - `value`: The NT_Value struct to get the type from.
    ///
    /// # Returns
    ///
    /// The type of the value, or unassigned if null.
    pub fn NT_GetValueType(value: *const NT_Value) -> NT_Type;

    /// Returns the boolean from the NT_Value.
    /// If the NT_Value is null, or is assigned to a different type, returns 0.
    ///
    /// # Parameters
    ///
    /// - `value`: NT_Value struct to get the boolean from.
    /// - `last_change`: Returns time in ms since the last change in the value.
    /// - `v_boolean`: Returns the boolean assigned to the name.
    ///
    /// # Returns
    ///
    /// 1 if successful, or 0 if value is null or not a boolean.
    pub fn NT_GetValueBoolean(
        value: *const NT_Value,
        last_change: *mut u64,
        v_boolean: *mut bool,
    ) -> NT_Bool;

    /// Returns the int from the NT_Value.
    /// If the NT_Value is null, or is assigned to a different type, returns 0.
    ///
    /// # Parameters
    ///
    /// - `value`: NT_Value struct to get the int from.
    /// - `last_change`: Returns time in ms since the last change in the value.
    /// - `v_int`: Returns the int assigned to the name.
    ///
    /// # Returns
    ///
    /// 1 if successful, or 0 if value is null or not an int.
    pub fn NT_GetValueInteger(
        value: *const NT_Value,
        last_change: *mut u64,
        v_int: *mut i64,
    ) -> NT_Bool;

    /// Returns the float from the NT_Value.
    /// If the NT_Value is null, or is assigned to a different type, returns 0.
    ///
    /// # Parameters
    ///
    /// - `value`: NT_Value struct to get the float from.
    /// - `last_change`: Returns time in ms since the last change in the value.
    /// - `v_float`: Returns the float assigned to the name.
    ///
    /// # Returns
    ///
    /// 1 if successful, or 0 if value is null or not a float.
    pub fn NT_GetValueFloat(
        value: *const NT_Value,
        last_change: *mut u64,
        v_float: *mut f32,
    ) -> NT_Bool;

    /// Returns the double from the NT_Value.
    /// If the NT_Value is null, or is assigned to a different type, returns 0.
    ///
    /// # Parameters
    ///
    /// - `value`: NT_Value struct to get the double from.
    /// - `last_change`: Returns time in ms since the last change in the value.
    /// - `v_double`: Returns the double assigned to the name.
    ///
    /// # Returns
    ///
    /// 1 if successful, or 0 if value is null or not a double.
    pub fn NT_GetValueDouble(
        value: *const NT_Value,
        last_change: *mut u64,
        v_double: *mut f64,
    ) -> NT_Bool;

    /// Returns a copy of the string from the NT_Value.
    /// If the NT_Value is null, or is assigned to a different type, returns 0.
    ///
    /// # Parameters
    ///
    /// - `value`: NT_Value struct to get the string from.
    /// - `last_change`: Returns time in ms since the last change in the value.
    /// - `str_len`: Returns the length of the string.
    ///
    /// # Returns
    ///
    /// Pointer to the string (UTF-8), or null if error.
    ///
    /// It is the caller's responsibility to free the string once it's no longer
    /// needed. The `NT_FreeCharArray()` function is useful for this purpose. The
    /// returned string is a copy of the string in the value, and must be freed
    /// separately.
    pub fn NT_GetValueString(
        value: *const NT_Value,
        last_change: *mut u64,
        str_len: *mut usize,
    ) -> *mut std::ffi::c_char;

    /// Returns a copy of the raw value from the NT_Value.
    /// If the NT_Value is null, or is assigned to a different type, returns null.
    ///
    /// # Parameters
    /// - `value`: NT_Value struct to get the string from.
    /// - `last_change`: Returns time in ms since the last change in the value.
    /// - `raw_len`: Returns the length of the string.
    ///
    /// # Returns
    /// Pointer to the raw value (UTF-8), or null if error.
    ///
    /// It is the caller's responsibility to free the raw value once it's no longer
    /// needed. The `NT_FreeCharArray()` function is useful for this purpose. The
    /// returned string is a copy of the string in the value, and must be freed
    /// separately.
    pub fn NT_GetValueRaw(
        value: *const NT_Value,
        last_change: *mut u64,
        raw_len: *mut usize,
    ) -> *mut u8;

    /// Returns a copy of the boolean array from the NT_Value.
    /// If the NT_Value is null, or is assigned to a different type, returns null.
    ///
    /// # Parameters
    /// - `value`: NT_Value struct to get the boolean array from.
    /// - `last_change`: Returns time in ms since the last change in the value.
    /// - `arr_size`: Returns the number of elements in the array.
    ///
    /// # Returns
    /// Pointer to the boolean array, or null if error.
    ///
    /// It is the caller's responsibility to free the array once it's no longer
    /// needed. The `NT_FreeBooleanArray()` function is useful for this purpose.
    /// The returned array is a copy of the array in the value, and must be
    /// freed separately.
    pub fn NT_GetValueBooleanArray(
        value: *const NT_Value,
        last_change: *mut u64,
        arr_size: *mut usize,
    ) -> *mut bool;

    /// Returns a copy of the int array from the NT_Value.
    /// If the NT_Value is null, or is assigned to a different type, returns null.
    ///
    /// # Parameters
    /// - `value`: NT_Value struct to get the int array from.
    /// - `last_change`: Returns time in ms since the last change in the value.
    /// - `arr_size`: Returns the number of elements in the array.
    ///
    /// # Returns
    /// Pointer to the int array, or null if error.
    ///
    /// It is the caller's responsibility to free the array once it's no longer
    /// needed. The `NT_FreeIntegerArray()` function is useful for this purpose.
    /// The returned array is a copy of the array in the value, and must be
    /// freed separately.
    pub fn NT_GetValueIntegerArray(
        value: *const NT_Value,
        last_change: *mut u64,
        arr_size: *mut usize,
    ) -> *mut i64;

    /// Returns a copy of the float array from the NT_Value.
    /// If the NT_Value is null, or is assigned to a different type, returns null.
    ///
    /// # Parameters
    /// - `value`: NT_Value struct to get the float array from.
    /// - `last_change`: Returns time in ms since the last change in the value.
    /// - `arr_size`: Returns the number of elements in the array.
    ///
    /// # Returns
    /// Pointer to the float array, or null if error.
    ///
    /// It is the caller's responsibility to free the array once it's no longer
    /// needed. The `NT_FreeFloatArray()` function is useful for this purpose.
    /// The returned array is a copy of the array in the value, and must be
    /// freed separately.
    pub fn NT_GetValueFloatArray(
        value: *const NT_Value,
        last_change: *mut u64,
        arr_size: *mut usize,
    ) -> *mut f32;

    /// Returns a copy of the double array from the NT_Value.
    /// If the NT_Value is null, or is assigned to a different type, returns null.
    ///
    /// # Parameters
    /// - `value`: NT_Value struct to get the double array from.
    /// - `last_change`: Returns time in ms since the last change in the value.
    /// - `arr_size`: Returns the number of elements in the array.
    ///
    /// # Returns
    /// Pointer to the double array, or null if error.
    ///
    /// It is the caller's responsibility to free the array once it's no longer
    /// needed. The `NT_FreeDoubleArray()` function is useful for this purpose.
    /// The returned array is a copy of the array in the value, and must be
    /// freed separately.
    pub fn NT_GetValueDoubleArray(
        value: *const NT_Value,
        last_change: *mut u64,
        arr_size: *mut usize,
    ) -> *mut f64;

    /// Returns a copy of the struct WPI_String array from the NT_Value.
    /// If the NT_Value is null, or is assigned to a different type, returns null.
    ///
    /// # Parameters
    /// - `value`: NT_Value struct to get the struct WPI_String array from.
    /// - `last_change`: Returns time in ms since the last change in the value.
    /// - `arr_size`: Returns the number of elements in the array.
    ///
    /// # Returns
    /// Pointer to the struct WPI_String array, or null if error.
    ///
    /// It is the caller's responsibility to free the array once it's no longer
    /// needed. The `WPI_FreeStringArray()` function is useful for this purpose.
    /// The returned array is a copy of the array in the value, and must be
    /// freed separately. Note that the individual struct WPI_Strings should not be
    /// freed, but the entire array should be freed at once. The
    /// `WPI_FreeStringArray()` function will free all the struct WPI_Strings.
    pub fn NT_GetValueStringArray(
        value: *const NT_Value,
        last_change: *mut u64,
        arr_size: *mut usize,
    ) -> *mut WPI_String;
}
