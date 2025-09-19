#[allow(unused_imports)]
use progenitor_client::{encode_path, ClientHooks, OperationInfo, RequestBuilderExt};
#[allow(unused_imports)]
pub use progenitor_client::{ByteStream, ClientInfo, Error, ResponseValue};
/// Types used as operation parameters and responses.
#[allow(clippy::all)]
pub mod types {
    /// Error types.
    pub mod error {
        /// Error from a `TryFrom` or `FromStr` implementation.
        pub struct ConversionError(::std::borrow::Cow<'static, str>);
        impl ::std::error::Error for ConversionError {}
        impl ::std::fmt::Display for ConversionError {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl ::std::fmt::Debug for ConversionError {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Debug::fmt(&self.0, f)
            }
        }

        impl From<&'static str> for ConversionError {
            fn from(value: &'static str) -> Self {
                Self(value.into())
            }
        }

        impl From<String> for ConversionError {
            fn from(value: String) -> Self {
                Self(value.into())
            }
        }
    }

    ///`AddEvent`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "event"
    ///  ],
    ///  "properties": {
    ///    "data": {
    ///      "description": "Event data",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "event": {
    ///      "description": "Name/type of the event",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct AddEvent {
        ///Event data
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub data: ::std::option::Option<::std::string::String>,
        ///Name/type of the event
        pub event: ::std::string::String,
    }

    impl ::std::convert::From<&AddEvent> for AddEvent {
        fn from(value: &AddEvent) -> Self {
            value.clone()
        }
    }

    impl AddEvent {
        pub fn builder() -> builder::AddEvent {
            Default::default()
        }
    }

    ///`AddEventsRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "events",
    ///    "key"
    ///  ],
    ///  "properties": {
    ///    "events": {
    ///      "description": "Events to add to the stream",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/AddEvent"
    ///      }
    ///    },
    ///    "key": {
    ///      "description": "Key of the stream to write to",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct AddEventsRequest {
        ///Events to add to the stream
        pub events: ::std::vec::Vec<AddEvent>,
        ///Key of the stream to write to
        pub key: ::std::string::String,
    }

    impl ::std::convert::From<&AddEventsRequest> for AddEventsRequest {
        fn from(value: &AddEventsRequest) -> Self {
            value.clone()
        }
    }

    impl AddEventsRequest {
        pub fn builder() -> builder::AddEventsRequest {
            Default::default()
        }
    }

    ///`AddEventsResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "ids"
    ///  ],
    ///  "properties": {
    ///    "ids": {
    ///      "description": "IDs of the added events",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct AddEventsResponse {
        ///IDs of the added events
        pub ids: ::std::vec::Vec<::std::string::String>,
    }

    impl ::std::convert::From<&AddEventsResponse> for AddEventsResponse {
        fn from(value: &AddEventsResponse) -> Self {
            value.clone()
        }
    }

    impl AddEventsResponse {
        pub fn builder() -> builder::AddEventsResponse {
            Default::default()
        }
    }

    ///`AddEventsStreamResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "errors",
    ///    "ids"
    ///  ],
    ///  "properties": {
    ///    "errors": {
    ///      "description": "Errors that occurred while adding events",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "ids": {
    ///      "description": "IDs of the added events",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct AddEventsStreamResponse {
        ///Errors that occurred while adding events
        pub errors: ::std::vec::Vec<::std::string::String>,
        ///IDs of the added events
        pub ids: ::std::vec::Vec<::std::string::String>,
    }

    impl ::std::convert::From<&AddEventsStreamResponse> for AddEventsStreamResponse {
        fn from(value: &AddEventsStreamResponse) -> Self {
            value.clone()
        }
    }

    impl AddEventsStreamResponse {
        pub fn builder() -> builder::AddEventsStreamResponse {
            Default::default()
        }
    }

    ///`EndStreamResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "status": {
    ///      "description": "Status of the stream",
    ///      "allOf": [
    ///        {
    ///          "$ref": "#/components/schemas/StreamStatus"
    ///        }
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct EndStreamResponse {
        ///Status of the stream
        pub status: StreamStatus,
    }

    impl ::std::convert::From<&EndStreamResponse> for EndStreamResponse {
        fn from(value: &EndStreamResponse) -> Self {
            value.clone()
        }
    }

    impl EndStreamResponse {
        pub fn builder() -> builder::EndStreamResponse {
            Default::default()
        }
    }

    ///`ErrorMessage`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "code",
    ///    "message"
    ///  ],
    ///  "properties": {
    ///    "code": {
    ///      "type": "string"
    ///    },
    ///    "message": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct ErrorMessage {
        pub code: ::std::string::String,
        pub message: ::std::string::String,
    }

    impl ::std::convert::From<&ErrorMessage> for ErrorMessage {
        fn from(value: &ErrorMessage) -> Self {
            value.clone()
        }
    }

    impl ErrorMessage {
        pub fn builder() -> builder::ErrorMessage {
            Default::default()
        }
    }

    ///`InfoResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "redis",
    ///    "url",
    ///    "version"
    ///  ],
    ///  "properties": {
    ///    "redis": {
    ///      "$ref": "#/components/schemas/RedisStats"
    ///    },
    ///    "url": {
    ///      "type": "string"
    ///    },
    ///    "version": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct InfoResponse {
        pub redis: RedisStats,
        pub url: ::std::string::String,
        pub version: ::std::string::String,
    }

    impl ::std::convert::From<&InfoResponse> for InfoResponse {
        fn from(value: &InfoResponse) -> Self {
            value.clone()
        }
    }

    impl InfoResponse {
        pub fn builder() -> builder::InfoResponse {
            Default::default()
        }
    }

    ///`RedisStats`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "static",
    ///    "streaming",
    ///    "streaming_available",
    ///    "streaming_in_use",
    ///    "streaming_max"
    ///  ],
    ///  "properties": {
    ///    "static": {
    ///      "description": "Number of static connections",
    ///      "type": "integer",
    ///      "format": "uint",
    ///      "minimum": 0.0
    ///    },
    ///    "streaming": {
    ///      "description": "Number of streaming connections",
    ///      "type": "integer",
    ///      "format": "uint",
    ///      "minimum": 0.0
    ///    },
    ///    "streaming_available": {
    ///      "description": "Number of available streaming connections",
    ///      "type": "integer",
    ///      "format": "uint",
    ///      "minimum": 0.0
    ///    },
    ///    "streaming_in_use": {
    ///      "description": "Number of in-use streaming connections",
    ///      "type": "integer",
    ///      "format": "uint",
    ///      "minimum": 0.0
    ///    },
    ///    "streaming_max": {
    ///      "description": "Maximum number of streaming connections",
    ///      "type": "integer",
    ///      "format": "uint",
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct RedisStats {
        ///Number of static connections
        #[serde(rename = "static")]
        pub static_: u32,
        ///Number of streaming connections
        pub streaming: u32,
        ///Number of available streaming connections
        pub streaming_available: u32,
        ///Number of in-use streaming connections
        pub streaming_in_use: u32,
        ///Maximum number of streaming connections
        pub streaming_max: u32,
    }

    impl ::std::convert::From<&RedisStats> for RedisStats {
        fn from(value: &RedisStats) -> Self {
            value.clone()
        }
    }

    impl RedisStats {
        pub fn builder() -> builder::RedisStats {
            Default::default()
        }
    }

    ///`StreamAccessResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "sse_url",
    ///    "token",
    ///    "ws_url"
    ///  ],
    ///  "properties": {
    ///    "sse_url": {
    ///      "description": "URL for the client to connect to the stream via
    /// SSE",
    ///      "type": "string"
    ///    },
    ///    "token": {
    ///      "description": "Client token to access the stream. Can be used as a
    /// Bearer token in the Authorization header (recommended) or as the `token`
    /// query parameter.",
    ///      "type": "string"
    ///    },
    ///    "ws_url": {
    ///      "description": "URL for the client to connect to the stream via
    /// WebSocket",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct StreamAccessResponse {
        ///URL for the client to connect to the stream via SSE
        pub sse_url: ::std::string::String,
        ///Client token to access the stream. Can be used as a Bearer token in
        /// the Authorization header (recommended) or as the `token` query
        /// parameter.
        pub token: ::std::string::String,
        ///URL for the client to connect to the stream via WebSocket
        pub ws_url: ::std::string::String,
    }

    impl ::std::convert::From<&StreamAccessResponse> for StreamAccessResponse {
        fn from(value: &StreamAccessResponse) -> Self {
            value.clone()
        }
    }

    impl StreamAccessResponse {
        pub fn builder() -> builder::StreamAccessResponse {
            Default::default()
        }
    }

    ///`StreamEvent`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "event",
    ///    "id",
    ///    "time"
    ///  ],
    ///  "properties": {
    ///    "data": {
    ///      "description": "Event data",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "event": {
    ///      "description": "Name/type of the event",
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "description": "ID of the event",
    ///      "type": "string"
    ///    },
    ///    "time": {
    ///      "description": "Time of the event (ISO 8601 format)",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct StreamEvent {
        ///Event data
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub data: ::std::option::Option<::std::string::String>,
        ///Name/type of the event
        pub event: ::std::string::String,
        ///ID of the event
        pub id: ::std::string::String,
        ///Time of the event (ISO 8601 format)
        pub time: ::std::string::String,
    }

    impl ::std::convert::From<&StreamEvent> for StreamEvent {
        fn from(value: &StreamEvent) -> Self {
            value.clone()
        }
    }

    impl StreamEvent {
        pub fn builder() -> builder::StreamEvent {
            Default::default()
        }
    }

    ///Information about the stream
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Information about the stream",
    ///  "type": "object",
    ///  "required": [
    ///    "key",
    ///    "length",
    ///    "ttl"
    ///  ],
    ///  "properties": {
    ///    "key": {
    ///      "description": "Key of the stream in Redis",
    ///      "type": "string"
    ///    },
    ///    "length": {
    ///      "description": "Number of events in the stream",
    ///      "type": "integer",
    ///      "format": "uint64",
    ///      "minimum": 0.0
    ///    },
    ///    "ttl": {
    ///      "description": "Expiration of the stream",
    ///      "type": "integer",
    ///      "format": "int64"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct StreamInfo {
        ///Key of the stream in Redis
        pub key: ::std::string::String,
        ///Number of events in the stream
        pub length: u64,
        ///Expiration of the stream
        pub ttl: i64,
    }

    impl ::std::convert::From<&StreamInfo> for StreamInfo {
        fn from(value: &StreamInfo) -> Self {
            value.clone()
        }
    }

    impl StreamInfo {
        pub fn builder() -> builder::StreamInfo {
            Default::default()
        }
    }

    ///`StreamRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "key"
    ///  ],
    ///  "properties": {
    ///    "key": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
    pub struct StreamRequest {
        pub key: ::std::string::String,
    }

    impl ::std::convert::From<&StreamRequest> for StreamRequest {
        fn from(value: &StreamRequest) -> Self {
            value.clone()
        }
    }

    impl StreamRequest {
        pub fn builder() -> builder::StreamRequest {
            Default::default()
        }
    }

    ///`StreamStatus`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "active",
    ///    "cancelled",
    ///    "ended"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        :: serde :: Deserialize,
        :: serde :: Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
    )]
    pub enum StreamStatus {
        #[serde(rename = "active")]
        Active,
        #[serde(rename = "cancelled")]
        Cancelled,
        #[serde(rename = "ended")]
        Ended,
    }

    impl ::std::convert::From<&Self> for StreamStatus {
        fn from(value: &StreamStatus) -> Self {
            value.clone()
        }
    }

    impl ::std::fmt::Display for StreamStatus {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Active => f.write_str("active"),
                Self::Cancelled => f.write_str("cancelled"),
                Self::Ended => f.write_str("ended"),
            }
        }
    }

    impl ::std::str::FromStr for StreamStatus {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "active" => Ok(Self::Active),
                "cancelled" => Ok(Self::Cancelled),
                "ended" => Ok(Self::Ended),
                _ => Err("invalid value".into()),
            }
        }
    }

    impl ::std::convert::TryFrom<&str> for StreamStatus {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<&::std::string::String> for StreamStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    impl ::std::convert::TryFrom<::std::string::String> for StreamStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }

    /// Types for composing complex structures.
    pub mod builder {
        #[derive(Clone, Debug)]
        pub struct AddEvent {
            data: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            event: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for AddEvent {
            fn default() -> Self {
                Self {
                    data: Ok(Default::default()),
                    event: Err("no value supplied for event".to_string()),
                }
            }
        }

        impl AddEvent {
            pub fn data<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.data = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for data: {}", e));
                self
            }
            pub fn event<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.event = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for event: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<AddEvent> for super::AddEvent {
            type Error = super::error::ConversionError;
            fn try_from(
                value: AddEvent,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    data: value.data?,
                    event: value.event?,
                })
            }
        }

        impl ::std::convert::From<super::AddEvent> for AddEvent {
            fn from(value: super::AddEvent) -> Self {
                Self {
                    data: Ok(value.data),
                    event: Ok(value.event),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct AddEventsRequest {
            events: ::std::result::Result<::std::vec::Vec<super::AddEvent>, ::std::string::String>,
            key: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for AddEventsRequest {
            fn default() -> Self {
                Self {
                    events: Err("no value supplied for events".to_string()),
                    key: Err("no value supplied for key".to_string()),
                }
            }
        }

        impl AddEventsRequest {
            pub fn events<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<super::AddEvent>>,
                T::Error: ::std::fmt::Display,
            {
                self.events = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for events: {}", e));
                self
            }
            pub fn key<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.key = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for key: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<AddEventsRequest> for super::AddEventsRequest {
            type Error = super::error::ConversionError;
            fn try_from(
                value: AddEventsRequest,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    events: value.events?,
                    key: value.key?,
                })
            }
        }

        impl ::std::convert::From<super::AddEventsRequest> for AddEventsRequest {
            fn from(value: super::AddEventsRequest) -> Self {
                Self {
                    events: Ok(value.events),
                    key: Ok(value.key),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct AddEventsResponse {
            ids: ::std::result::Result<
                ::std::vec::Vec<::std::string::String>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for AddEventsResponse {
            fn default() -> Self {
                Self {
                    ids: Err("no value supplied for ids".to_string()),
                }
            }
        }

        impl AddEventsResponse {
            pub fn ids<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.ids = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for ids: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<AddEventsResponse> for super::AddEventsResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: AddEventsResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self { ids: value.ids? })
            }
        }

        impl ::std::convert::From<super::AddEventsResponse> for AddEventsResponse {
            fn from(value: super::AddEventsResponse) -> Self {
                Self { ids: Ok(value.ids) }
            }
        }

        #[derive(Clone, Debug)]
        pub struct AddEventsStreamResponse {
            errors: ::std::result::Result<
                ::std::vec::Vec<::std::string::String>,
                ::std::string::String,
            >,
            ids: ::std::result::Result<
                ::std::vec::Vec<::std::string::String>,
                ::std::string::String,
            >,
        }

        impl ::std::default::Default for AddEventsStreamResponse {
            fn default() -> Self {
                Self {
                    errors: Err("no value supplied for errors".to_string()),
                    ids: Err("no value supplied for ids".to_string()),
                }
            }
        }

        impl AddEventsStreamResponse {
            pub fn errors<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.errors = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for errors: {}", e));
                self
            }
            pub fn ids<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::vec::Vec<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.ids = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for ids: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<AddEventsStreamResponse> for super::AddEventsStreamResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: AddEventsStreamResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    errors: value.errors?,
                    ids: value.ids?,
                })
            }
        }

        impl ::std::convert::From<super::AddEventsStreamResponse> for AddEventsStreamResponse {
            fn from(value: super::AddEventsStreamResponse) -> Self {
                Self {
                    errors: Ok(value.errors),
                    ids: Ok(value.ids),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct EndStreamResponse {
            status: ::std::result::Result<super::StreamStatus, ::std::string::String>,
        }

        impl ::std::default::Default for EndStreamResponse {
            fn default() -> Self {
                Self {
                    status: Err("no value supplied for status".to_string()),
                }
            }
        }

        impl EndStreamResponse {
            pub fn status<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::StreamStatus>,
                T::Error: ::std::fmt::Display,
            {
                self.status = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for status: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<EndStreamResponse> for super::EndStreamResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: EndStreamResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    status: value.status?,
                })
            }
        }

        impl ::std::convert::From<super::EndStreamResponse> for EndStreamResponse {
            fn from(value: super::EndStreamResponse) -> Self {
                Self {
                    status: Ok(value.status),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct ErrorMessage {
            code: ::std::result::Result<::std::string::String, ::std::string::String>,
            message: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for ErrorMessage {
            fn default() -> Self {
                Self {
                    code: Err("no value supplied for code".to_string()),
                    message: Err("no value supplied for message".to_string()),
                }
            }
        }

        impl ErrorMessage {
            pub fn code<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.code = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for code: {}", e));
                self
            }
            pub fn message<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.message = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for message: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<ErrorMessage> for super::ErrorMessage {
            type Error = super::error::ConversionError;
            fn try_from(
                value: ErrorMessage,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    code: value.code?,
                    message: value.message?,
                })
            }
        }

        impl ::std::convert::From<super::ErrorMessage> for ErrorMessage {
            fn from(value: super::ErrorMessage) -> Self {
                Self {
                    code: Ok(value.code),
                    message: Ok(value.message),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct InfoResponse {
            redis: ::std::result::Result<super::RedisStats, ::std::string::String>,
            url: ::std::result::Result<::std::string::String, ::std::string::String>,
            version: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for InfoResponse {
            fn default() -> Self {
                Self {
                    redis: Err("no value supplied for redis".to_string()),
                    url: Err("no value supplied for url".to_string()),
                    version: Err("no value supplied for version".to_string()),
                }
            }
        }

        impl InfoResponse {
            pub fn redis<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<super::RedisStats>,
                T::Error: ::std::fmt::Display,
            {
                self.redis = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for redis: {}", e));
                self
            }
            pub fn url<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.url = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for url: {}", e));
                self
            }
            pub fn version<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.version = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for version: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<InfoResponse> for super::InfoResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: InfoResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    redis: value.redis?,
                    url: value.url?,
                    version: value.version?,
                })
            }
        }

        impl ::std::convert::From<super::InfoResponse> for InfoResponse {
            fn from(value: super::InfoResponse) -> Self {
                Self {
                    redis: Ok(value.redis),
                    url: Ok(value.url),
                    version: Ok(value.version),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct RedisStats {
            static_: ::std::result::Result<u32, ::std::string::String>,
            streaming: ::std::result::Result<u32, ::std::string::String>,
            streaming_available: ::std::result::Result<u32, ::std::string::String>,
            streaming_in_use: ::std::result::Result<u32, ::std::string::String>,
            streaming_max: ::std::result::Result<u32, ::std::string::String>,
        }

        impl ::std::default::Default for RedisStats {
            fn default() -> Self {
                Self {
                    static_: Err("no value supplied for static_".to_string()),
                    streaming: Err("no value supplied for streaming".to_string()),
                    streaming_available: Err(
                        "no value supplied for streaming_available".to_string()
                    ),
                    streaming_in_use: Err("no value supplied for streaming_in_use".to_string()),
                    streaming_max: Err("no value supplied for streaming_max".to_string()),
                }
            }
        }

        impl RedisStats {
            pub fn static_<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u32>,
                T::Error: ::std::fmt::Display,
            {
                self.static_ = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for static_: {}", e));
                self
            }
            pub fn streaming<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u32>,
                T::Error: ::std::fmt::Display,
            {
                self.streaming = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for streaming: {}", e));
                self
            }
            pub fn streaming_available<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u32>,
                T::Error: ::std::fmt::Display,
            {
                self.streaming_available = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for streaming_available: {}",
                        e
                    )
                });
                self
            }
            pub fn streaming_in_use<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u32>,
                T::Error: ::std::fmt::Display,
            {
                self.streaming_in_use = value.try_into().map_err(|e| {
                    format!(
                        "error converting supplied value for streaming_in_use: {}",
                        e
                    )
                });
                self
            }
            pub fn streaming_max<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u32>,
                T::Error: ::std::fmt::Display,
            {
                self.streaming_max = value.try_into().map_err(|e| {
                    format!("error converting supplied value for streaming_max: {}", e)
                });
                self
            }
        }

        impl ::std::convert::TryFrom<RedisStats> for super::RedisStats {
            type Error = super::error::ConversionError;
            fn try_from(
                value: RedisStats,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    static_: value.static_?,
                    streaming: value.streaming?,
                    streaming_available: value.streaming_available?,
                    streaming_in_use: value.streaming_in_use?,
                    streaming_max: value.streaming_max?,
                })
            }
        }

        impl ::std::convert::From<super::RedisStats> for RedisStats {
            fn from(value: super::RedisStats) -> Self {
                Self {
                    static_: Ok(value.static_),
                    streaming: Ok(value.streaming),
                    streaming_available: Ok(value.streaming_available),
                    streaming_in_use: Ok(value.streaming_in_use),
                    streaming_max: Ok(value.streaming_max),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct StreamAccessResponse {
            sse_url: ::std::result::Result<::std::string::String, ::std::string::String>,
            token: ::std::result::Result<::std::string::String, ::std::string::String>,
            ws_url: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for StreamAccessResponse {
            fn default() -> Self {
                Self {
                    sse_url: Err("no value supplied for sse_url".to_string()),
                    token: Err("no value supplied for token".to_string()),
                    ws_url: Err("no value supplied for ws_url".to_string()),
                }
            }
        }

        impl StreamAccessResponse {
            pub fn sse_url<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.sse_url = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for sse_url: {}", e));
                self
            }
            pub fn token<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.token = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for token: {}", e));
                self
            }
            pub fn ws_url<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.ws_url = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for ws_url: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<StreamAccessResponse> for super::StreamAccessResponse {
            type Error = super::error::ConversionError;
            fn try_from(
                value: StreamAccessResponse,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    sse_url: value.sse_url?,
                    token: value.token?,
                    ws_url: value.ws_url?,
                })
            }
        }

        impl ::std::convert::From<super::StreamAccessResponse> for StreamAccessResponse {
            fn from(value: super::StreamAccessResponse) -> Self {
                Self {
                    sse_url: Ok(value.sse_url),
                    token: Ok(value.token),
                    ws_url: Ok(value.ws_url),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct StreamEvent {
            data: ::std::result::Result<
                ::std::option::Option<::std::string::String>,
                ::std::string::String,
            >,
            event: ::std::result::Result<::std::string::String, ::std::string::String>,
            id: ::std::result::Result<::std::string::String, ::std::string::String>,
            time: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for StreamEvent {
            fn default() -> Self {
                Self {
                    data: Ok(Default::default()),
                    event: Err("no value supplied for event".to_string()),
                    id: Err("no value supplied for id".to_string()),
                    time: Err("no value supplied for time".to_string()),
                }
            }
        }

        impl StreamEvent {
            pub fn data<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::option::Option<::std::string::String>>,
                T::Error: ::std::fmt::Display,
            {
                self.data = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for data: {}", e));
                self
            }
            pub fn event<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.event = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for event: {}", e));
                self
            }
            pub fn id<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.id = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for id: {}", e));
                self
            }
            pub fn time<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.time = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for time: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<StreamEvent> for super::StreamEvent {
            type Error = super::error::ConversionError;
            fn try_from(
                value: StreamEvent,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    data: value.data?,
                    event: value.event?,
                    id: value.id?,
                    time: value.time?,
                })
            }
        }

        impl ::std::convert::From<super::StreamEvent> for StreamEvent {
            fn from(value: super::StreamEvent) -> Self {
                Self {
                    data: Ok(value.data),
                    event: Ok(value.event),
                    id: Ok(value.id),
                    time: Ok(value.time),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct StreamInfo {
            key: ::std::result::Result<::std::string::String, ::std::string::String>,
            length: ::std::result::Result<u64, ::std::string::String>,
            ttl: ::std::result::Result<i64, ::std::string::String>,
        }

        impl ::std::default::Default for StreamInfo {
            fn default() -> Self {
                Self {
                    key: Err("no value supplied for key".to_string()),
                    length: Err("no value supplied for length".to_string()),
                    ttl: Err("no value supplied for ttl".to_string()),
                }
            }
        }

        impl StreamInfo {
            pub fn key<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.key = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for key: {}", e));
                self
            }
            pub fn length<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<u64>,
                T::Error: ::std::fmt::Display,
            {
                self.length = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for length: {}", e));
                self
            }
            pub fn ttl<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<i64>,
                T::Error: ::std::fmt::Display,
            {
                self.ttl = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for ttl: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<StreamInfo> for super::StreamInfo {
            type Error = super::error::ConversionError;
            fn try_from(
                value: StreamInfo,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self {
                    key: value.key?,
                    length: value.length?,
                    ttl: value.ttl?,
                })
            }
        }

        impl ::std::convert::From<super::StreamInfo> for StreamInfo {
            fn from(value: super::StreamInfo) -> Self {
                Self {
                    key: Ok(value.key),
                    length: Ok(value.length),
                    ttl: Ok(value.ttl),
                }
            }
        }

        #[derive(Clone, Debug)]
        pub struct StreamRequest {
            key: ::std::result::Result<::std::string::String, ::std::string::String>,
        }

        impl ::std::default::Default for StreamRequest {
            fn default() -> Self {
                Self {
                    key: Err("no value supplied for key".to_string()),
                }
            }
        }

        impl StreamRequest {
            pub fn key<T>(mut self, value: T) -> Self
            where
                T: ::std::convert::TryInto<::std::string::String>,
                T::Error: ::std::fmt::Display,
            {
                self.key = value
                    .try_into()
                    .map_err(|e| format!("error converting supplied value for key: {}", e));
                self
            }
        }

        impl ::std::convert::TryFrom<StreamRequest> for super::StreamRequest {
            type Error = super::error::ConversionError;
            fn try_from(
                value: StreamRequest,
            ) -> ::std::result::Result<Self, super::error::ConversionError> {
                Ok(Self { key: value.key? })
            }
        }

        impl ::std::convert::From<super::StreamRequest> for StreamRequest {
            fn from(value: super::StreamRequest) -> Self {
                Self { key: Ok(value.key) }
            }
        }
    }
}

#[derive(Clone, Debug)]
///Client for tinistream
///
///Version: 0.1.5
pub struct Client {
    pub(crate) baseurl: String,
    pub(crate) client: reqwest::Client,
}

impl Client {
    /// Create a new client.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new(baseurl: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = {
            let dur = std::time::Duration::from_secs(15);
            reqwest::ClientBuilder::new()
                .connect_timeout(dur)
                .timeout(dur)
        };
        #[cfg(target_arch = "wasm32")]
        let client = reqwest::ClientBuilder::new();
        Self::new_with_client(baseurl, client.build().unwrap())
    }

    /// Construct a new client with an existing `reqwest::Client`,
    /// allowing more control over its configuration.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new_with_client(baseurl: &str, client: reqwest::Client) -> Self {
        Self {
            baseurl: baseurl.to_string(),
            client,
        }
    }
}

impl ClientInfo<()> for Client {
    fn api_version() -> &'static str {
        "0.1.5"
    }

    fn baseurl(&self) -> &str {
        self.baseurl.as_str()
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    fn inner(&self) -> &() {
        &()
    }
}

impl ClientHooks<()> for &Client {}
pub trait ClientClientExt {
    ///Connect SSE
    ///
    ///Connect to a stream and receive SSE events
    ///
    ///Sends a `GET` request to `/api/client/sse`
    ///
    ///```ignore
    /// let response = client.connect_sse()
    ///    .key(key)
    ///    .send()
    ///    .await;
    /// ```
    fn connect_sse(&self) -> builder::ConnectSse<'_>;
}

impl ClientClientExt for Client {
    fn connect_sse(&self) -> builder::ConnectSse<'_> {
        builder::ConnectSse::new(self)
    }
}

pub trait ClientEventsExt {
    ///Add events
    ///
    ///Add events to a stream
    ///
    ///Sends a `POST` request to `/api/event/add`
    ///
    ///```ignore
    /// let response = client.add_events()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    fn add_events(&self) -> builder::AddEvents<'_>;
    ///Add events JSON stream
    ///
    ///Add events to a stream via a JSON stream. Events are sent as
    /// newline-delimited JSON objects.
    ///
    ///Sends a `POST` request to `/api/event/add/json-stream`
    ///
    ///Arguments:
    /// - `key`
    /// - `body`: JSON Lines (stream of JSON strings separated by newlines)
    ///```ignore
    /// let response = client.add_events_json_stream()
    ///    .key(key)
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    fn add_events_json_stream(&self) -> builder::AddEventsJsonStream<'_>;
}

impl ClientEventsExt for Client {
    fn add_events(&self) -> builder::AddEvents<'_> {
        builder::AddEvents::new(self)
    }

    fn add_events_json_stream(&self) -> builder::AddEventsJsonStream<'_> {
        builder::AddEventsJsonStream::new(self)
    }
}

pub trait ClientInfoExt {
    ///Health check
    ///
    ///Sends a `GET` request to `/api/health`
    ///
    ///```ignore
    /// let response = client.health()
    ///    .send()
    ///    .await;
    /// ```
    fn health(&self) -> builder::Health<'_>;
    ///Get info
    ///
    ///Get information about the server
    ///
    ///Sends a `GET` request to `/api/info`
    ///
    ///```ignore
    /// let response = client.get_info()
    ///    .send()
    ///    .await;
    /// ```
    fn get_info(&self) -> builder::GetInfo<'_>;
}

impl ClientInfoExt for Client {
    fn health(&self) -> builder::Health<'_> {
        builder::Health::new(self)
    }

    fn get_info(&self) -> builder::GetInfo<'_> {
        builder::GetInfo::new(self)
    }
}

pub trait ClientStreamExt {
    ///List streams
    ///
    ///List all active streams
    ///
    ///Sends a `GET` request to `/api/stream/`
    ///
    ///```ignore
    /// let response = client.list_streams()
    ///    .pattern(pattern)
    ///    .send()
    ///    .await;
    /// ```
    fn list_streams(&self) -> builder::ListStreams<'_>;
    ///Create stream
    ///
    ///Create a new stream, and get a client URL and token to connect to the
    /// stream
    ///
    ///Sends a `POST` request to `/api/stream/`
    ///
    ///```ignore
    /// let response = client.create_stream()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    fn create_stream(&self) -> builder::CreateStream<'_>;
    ///Create stream token
    ///
    ///Create a new client token to connect to a stream
    ///
    ///Sends a `POST` request to `/api/stream/token`
    ///
    ///```ignore
    /// let response = client.create_token()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    fn create_token(&self) -> builder::CreateToken<'_>;
    ///Get stream info
    ///
    ///Get info on an active stream
    ///
    ///Sends a `GET` request to `/api/stream/info`
    ///
    ///```ignore
    /// let response = client.get_stream_info()
    ///    .key(key)
    ///    .send()
    ///    .await;
    /// ```
    fn get_stream_info(&self) -> builder::GetStreamInfo<'_>;
    ///Get stream events
    ///
    ///Get all events so far in a stream
    ///
    ///Sends a `GET` request to `/api/stream/events`
    ///
    ///```ignore
    /// let response = client.get_stream_events()
    ///    .key(key)
    ///    .send()
    ///    .await;
    /// ```
    fn get_stream_events(&self) -> builder::GetStreamEvents<'_>;
    ///Cancel stream
    ///
    ///Sends a `POST` request to `/api/stream/cancel`
    ///
    ///```ignore
    /// let response = client.cancel_stream()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    fn cancel_stream(&self) -> builder::CancelStream<'_>;
    ///End stream
    ///
    ///Sends a `POST` request to `/api/stream/end`
    ///
    ///```ignore
    /// let response = client.end_stream()
    ///    .body(body)
    ///    .send()
    ///    .await;
    /// ```
    fn end_stream(&self) -> builder::EndStream<'_>;
}

impl ClientStreamExt for Client {
    fn list_streams(&self) -> builder::ListStreams<'_> {
        builder::ListStreams::new(self)
    }

    fn create_stream(&self) -> builder::CreateStream<'_> {
        builder::CreateStream::new(self)
    }

    fn create_token(&self) -> builder::CreateToken<'_> {
        builder::CreateToken::new(self)
    }

    fn get_stream_info(&self) -> builder::GetStreamInfo<'_> {
        builder::GetStreamInfo::new(self)
    }

    fn get_stream_events(&self) -> builder::GetStreamEvents<'_> {
        builder::GetStreamEvents::new(self)
    }

    fn cancel_stream(&self) -> builder::CancelStream<'_> {
        builder::CancelStream::new(self)
    }

    fn end_stream(&self) -> builder::EndStream<'_> {
        builder::EndStream::new(self)
    }
}

/// Types for composing operation parameters.
#[allow(clippy::all)]
pub mod builder {
    use super::types;
    #[allow(unused_imports)]
    use super::{
        encode_path, ByteStream, ClientHooks, ClientInfo, Error, OperationInfo, RequestBuilderExt,
        ResponseValue,
    };
    ///Builder for [`ClientInfoExt::health`]
    ///
    ///[`ClientInfoExt::health`]: super::ClientInfoExt::health
    #[derive(Debug, Clone)]
    pub struct Health<'a> {
        client: &'a super::Client,
    }

    impl<'a> Health<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `GET` request to `/api/health`
        pub async fn send(self) -> Result<ResponseValue<ByteStream>, Error<()>> {
            let Self { client } = self;
            let url = format!("{}/api/health", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client.client.get(url).headers(header_map).build()?;
            let info = OperationInfo {
                operation_id: "health",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => Ok(ResponseValue::stream(response)),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`ClientInfoExt::get_info`]
    ///
    ///[`ClientInfoExt::get_info`]: super::ClientInfoExt::get_info
    #[derive(Debug, Clone)]
    pub struct GetInfo<'a> {
        client: &'a super::Client,
    }

    impl<'a> GetInfo<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self { client: client }
        }

        ///Sends a `GET` request to `/api/info`
        pub async fn send(self) -> Result<ResponseValue<types::InfoResponse>, Error<()>> {
            let Self { client } = self;
            let url = format!("{}/api/info", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "get_info",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`ClientClientExt::connect_sse`]
    ///
    ///[`ClientClientExt::connect_sse`]: super::ClientClientExt::connect_sse
    #[derive(Debug, Clone)]
    pub struct ConnectSse<'a> {
        client: &'a super::Client,
        key: Result<::std::string::String, String>,
    }

    impl<'a> ConnectSse<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                key: Err("key was not initialized".to_string()),
            }
        }

        pub fn key<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.key = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for key failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/api/client/sse`
        pub async fn send(self) -> Result<ResponseValue<ByteStream>, Error<types::ErrorMessage>> {
            let Self { client, key } = self;
            let key = key.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/client/sse", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .query(&progenitor_client::QueryParam::new("key", &key))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "connect_sse",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => Ok(ResponseValue::stream(response)),
                400u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                401u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                404u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                422u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                500u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`ClientStreamExt::list_streams`]
    ///
    ///[`ClientStreamExt::list_streams`]: super::ClientStreamExt::list_streams
    #[derive(Debug, Clone)]
    pub struct ListStreams<'a> {
        client: &'a super::Client,
        pattern: Result<Option<::std::string::String>, String>,
    }

    impl<'a> ListStreams<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                pattern: Ok(None),
            }
        }

        pub fn pattern<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.pattern = value.try_into().map(Some).map_err(|_| {
                "conversion to `:: std :: string :: String` for pattern failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/api/stream/`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<::std::vec::Vec<types::StreamInfo>>, Error<types::ErrorMessage>>
        {
            let Self { client, pattern } = self;
            let pattern = pattern.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/stream/", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new("pattern", &pattern))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "list_streams",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                400u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                401u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                404u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                422u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                500u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`ClientStreamExt::create_stream`]
    ///
    ///[`ClientStreamExt::create_stream`]: super::ClientStreamExt::create_stream
    #[derive(Debug, Clone)]
    pub struct CreateStream<'a> {
        client: &'a super::Client,
        body: Result<types::builder::StreamRequest, String>,
    }

    impl<'a> CreateStream<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::StreamRequest>,
            <V as std::convert::TryInto<types::StreamRequest>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `StreamRequest` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(types::builder::StreamRequest) -> types::builder::StreamRequest,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/api/stream/`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<types::StreamAccessResponse>, Error<types::ErrorMessage>>
        {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| types::StreamRequest::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/stream/", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "create_stream",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                400u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                401u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                404u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                422u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                500u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`ClientStreamExt::create_token`]
    ///
    ///[`ClientStreamExt::create_token`]: super::ClientStreamExt::create_token
    #[derive(Debug, Clone)]
    pub struct CreateToken<'a> {
        client: &'a super::Client,
        body: Result<types::builder::StreamRequest, String>,
    }

    impl<'a> CreateToken<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::StreamRequest>,
            <V as std::convert::TryInto<types::StreamRequest>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `StreamRequest` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(types::builder::StreamRequest) -> types::builder::StreamRequest,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/api/stream/token`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<types::StreamAccessResponse>, Error<types::ErrorMessage>>
        {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| types::StreamRequest::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/stream/token", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "create_token",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                400u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                401u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                404u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                422u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                500u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`ClientStreamExt::get_stream_info`]
    ///
    ///[`ClientStreamExt::get_stream_info`]: super::ClientStreamExt::get_stream_info
    #[derive(Debug, Clone)]
    pub struct GetStreamInfo<'a> {
        client: &'a super::Client,
        key: Result<::std::string::String, String>,
    }

    impl<'a> GetStreamInfo<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                key: Err("key was not initialized".to_string()),
            }
        }

        pub fn key<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.key = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for key failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/api/stream/info`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<types::StreamInfo>, Error<types::ErrorMessage>> {
            let Self { client, key } = self;
            let key = key.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/stream/info", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new("key", &key))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "get_stream_info",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                400u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                401u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                404u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                422u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                500u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`ClientStreamExt::get_stream_events`]
    ///
    ///[`ClientStreamExt::get_stream_events`]: super::ClientStreamExt::get_stream_events
    #[derive(Debug, Clone)]
    pub struct GetStreamEvents<'a> {
        client: &'a super::Client,
        key: Result<::std::string::String, String>,
    }

    impl<'a> GetStreamEvents<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                key: Err("key was not initialized".to_string()),
            }
        }

        pub fn key<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.key = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for key failed".to_string()
            });
            self
        }

        ///Sends a `GET` request to `/api/stream/events`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<::std::vec::Vec<types::StreamEvent>>, Error<types::ErrorMessage>>
        {
            let Self { client, key } = self;
            let key = key.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/stream/events", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .get(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .query(&progenitor_client::QueryParam::new("key", &key))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "get_stream_events",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                400u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                401u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                404u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                422u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                500u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`ClientStreamExt::cancel_stream`]
    ///
    ///[`ClientStreamExt::cancel_stream`]: super::ClientStreamExt::cancel_stream
    #[derive(Debug, Clone)]
    pub struct CancelStream<'a> {
        client: &'a super::Client,
        body: Result<types::builder::StreamRequest, String>,
    }

    impl<'a> CancelStream<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::StreamRequest>,
            <V as std::convert::TryInto<types::StreamRequest>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `StreamRequest` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(types::builder::StreamRequest) -> types::builder::StreamRequest,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/api/stream/cancel`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<types::EndStreamResponse>, Error<types::ErrorMessage>> {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| types::StreamRequest::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/stream/cancel", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "cancel_stream",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                400u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                401u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                404u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                422u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                500u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`ClientStreamExt::end_stream`]
    ///
    ///[`ClientStreamExt::end_stream`]: super::ClientStreamExt::end_stream
    #[derive(Debug, Clone)]
    pub struct EndStream<'a> {
        client: &'a super::Client,
        body: Result<types::builder::StreamRequest, String>,
    }

    impl<'a> EndStream<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::StreamRequest>,
            <V as std::convert::TryInto<types::StreamRequest>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `StreamRequest` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(types::builder::StreamRequest) -> types::builder::StreamRequest,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/api/stream/end`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<types::EndStreamResponse>, Error<types::ErrorMessage>> {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| types::StreamRequest::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/stream/end", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "end_stream",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                400u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                401u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                404u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                422u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                500u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`ClientEventsExt::add_events`]
    ///
    ///[`ClientEventsExt::add_events`]: super::ClientEventsExt::add_events
    #[derive(Debug, Clone)]
    pub struct AddEvents<'a> {
        client: &'a super::Client,
        body: Result<types::builder::AddEventsRequest, String>,
    }

    impl<'a> AddEvents<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                body: Ok(::std::default::Default::default()),
            }
        }

        pub fn body<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<types::AddEventsRequest>,
            <V as std::convert::TryInto<types::AddEventsRequest>>::Error: std::fmt::Display,
        {
            self.body = value
                .try_into()
                .map(From::from)
                .map_err(|s| format!("conversion to `AddEventsRequest` for body failed: {}", s));
            self
        }

        pub fn body_map<F>(mut self, f: F) -> Self
        where
            F: std::ops::FnOnce(
                types::builder::AddEventsRequest,
            ) -> types::builder::AddEventsRequest,
        {
            self.body = self.body.map(f);
            self
        }

        ///Sends a `POST` request to `/api/event/add`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<types::AddEventsResponse>, Error<types::ErrorMessage>> {
            let Self { client, body } = self;
            let body = body
                .and_then(|v| types::AddEventsRequest::try_from(v).map_err(|e| e.to_string()))
                .map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/event/add", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .json(&body)
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "add_events",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                400u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                401u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                404u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                422u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                500u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }

    ///Builder for [`ClientEventsExt::add_events_json_stream`]
    ///
    ///[`ClientEventsExt::add_events_json_stream`]: super::ClientEventsExt::add_events_json_stream
    #[derive(Debug)]
    pub struct AddEventsJsonStream<'a> {
        client: &'a super::Client,
        key: Result<::std::string::String, String>,
        body: Result<reqwest::Body, String>,
    }

    impl<'a> AddEventsJsonStream<'a> {
        pub fn new(client: &'a super::Client) -> Self {
            Self {
                client: client,
                key: Err("key was not initialized".to_string()),
                body: Err("body was not initialized".to_string()),
            }
        }

        pub fn key<V>(mut self, value: V) -> Self
        where
            V: std::convert::TryInto<::std::string::String>,
        {
            self.key = value.try_into().map_err(|_| {
                "conversion to `:: std :: string :: String` for key failed".to_string()
            });
            self
        }

        pub fn body<B>(mut self, value: B) -> Self
        where
            B: std::convert::TryInto<reqwest::Body>,
        {
            self.body = value
                .try_into()
                .map_err(|_| "conversion to `reqwest::Body` for body failed".to_string());
            self
        }

        ///Sends a `POST` request to `/api/event/add/json-stream`
        pub async fn send(
            self,
        ) -> Result<ResponseValue<types::AddEventsStreamResponse>, Error<types::ErrorMessage>>
        {
            let Self { client, key, body } = self;
            let key = key.map_err(Error::InvalidRequest)?;
            let body = body.map_err(Error::InvalidRequest)?;
            let url = format!("{}/api/event/add/json-stream", client.baseurl,);
            let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
            header_map.append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(super::Client::api_version()),
            );
            #[allow(unused_mut)]
            let mut request = client
                .client
                .post(url)
                .header(
                    ::reqwest::header::ACCEPT,
                    ::reqwest::header::HeaderValue::from_static("application/json"),
                )
                .header(
                    ::reqwest::header::CONTENT_TYPE,
                    ::reqwest::header::HeaderValue::from_static("application/octet-stream"),
                )
                .body(body)
                .query(&progenitor_client::QueryParam::new("key", &key))
                .headers(header_map)
                .build()?;
            let info = OperationInfo {
                operation_id: "add_events_json_stream",
            };
            client.pre(&mut request, &info).await?;
            let result = client.exec(request, &info).await;
            client.post(&result, &info).await?;
            let response = result?;
            match response.status().as_u16() {
                200u16 => ResponseValue::from_response(response).await,
                400u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                401u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                404u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                422u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                500u16 => Err(Error::ErrorResponse(
                    ResponseValue::from_response(response).await?,
                )),
                _ => Err(Error::UnexpectedResponse(response)),
            }
        }
    }
}

/// Items consumers will typically use such as the Client and
/// extension traits.
pub mod prelude {
    #[allow(unused_imports)]
    pub use super::Client;
    pub use super::ClientClientExt;
    pub use super::ClientEventsExt;
    pub use super::ClientInfoExt;
    pub use super::ClientStreamExt;
}
