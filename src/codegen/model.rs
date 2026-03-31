use endpoint_libs::libs::error_code::ErrorCode;
use endpoint_libs::libs::types::*;
use endpoint_libs::libs::ws::*;
use num_derive::FromPrimitive;
use rkyv::Archive;
use serde::*;
use std::net::IpAddr;
use strum_macros::{Display, EnumString};
use uuid::Uuid;
use worktable::prelude::*;

#[derive(
    MemStat,
    Archive,
    Clone,
    Copy,
    Debug,
    Display,
    PartialEq,
    PartialOrd,
    Eq,
    Hash,
    Ord,
    EnumString,
    rkyv::Deserialize,
    rkyv::Serialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[repr(u8)]
pub enum UserRole {
    ///
    Public = 0,
    ///
    Authorized = 1,
    ///
    Admin = 2,
    /// This role is used exclusively for connections from the Honey-Auth BE to App BEs for comms such as token callbacks
    HoneyAuth = 6,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SupportMessage {
    pub incoming: bool,
    pub sent_by: String,
    pub sent_at: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserView {
    pub user_public_id: Uuid,
    pub username: String,
    pub role: UserRole,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WaitlistLead {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub telegram: Option<String>,
    #[serde(default)]
    pub whatsApp: Option<String>,
    pub description: String,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, FromPrimitive, PartialEq, Eq, PartialOrd, Ord, EnumString, Display, Hash,
)]
pub enum EnumEndpoint {
    ///
    Init = 11000,
    ///
    SetRole = 21000,
    ///
    ListUsers = 21001,
    ///
    AddSupports = 21002,
    ///
    RemoveSupports = 21003,
    ///
    ListSupports = 21004,
    ///
    SendMsg = 31000,
    ///
    ListMsgs = 31001,
    ///
    SubMsgEvents = 31002,
    ///
    ListLeads = 41000,
    ///
    AddLead = 41001,
}

impl EnumEndpoint {
    pub fn schema(&self) -> endpoint_libs::model::EndpointSchema {
        let schema = match self {
            Self::Init => InitRequest::SCHEMA,
            Self::SetRole => SetRoleRequest::SCHEMA,
            Self::ListUsers => ListUsersRequest::SCHEMA,
            Self::AddSupports => AddSupportsRequest::SCHEMA,
            Self::RemoveSupports => RemoveSupportsRequest::SCHEMA,
            Self::ListSupports => ListSupportsRequest::SCHEMA,
            Self::SendMsg => SendMsgRequest::SCHEMA,
            Self::ListMsgs => ListMsgsRequest::SCHEMA,
            Self::SubMsgEvents => SubMsgEventsRequest::SCHEMA,
            Self::ListLeads => ListLeadsRequest::SCHEMA,
            Self::AddLead => AddLeadRequest::SCHEMA,
        };
        serde_json::from_str(schema).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ErrorXxx {}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, FromPrimitive, PartialEq, Eq, PartialOrd, Ord, EnumString, Display, Hash,
)]
pub enum EnumErrorCode {
    /// None Please populate error_codes.json
    Xxx = 0,
}

impl From<EnumErrorCode> for ErrorCode {
    fn from(e: EnumErrorCode) -> Self {
        ErrorCode::new(e as _)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddLeadRequest {
    pub name: String,
    #[serde(default)]
    pub telegram: Option<String>,
    #[serde(default)]
    pub whatsApp: Option<String>,
    pub description: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddLeadResponse {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddSupportsRequest {
    pub tg_handles: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddSupportsResponse {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InitRequest {
    pub access_token: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InitResponse {
    pub user_public_id: Uuid,
    pub role: UserRole,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListLeadsRequest {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListLeadsResponse {
    pub data: Vec<WaitlistLead>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListMsgsRequest {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListMsgsResponse {
    pub data: Vec<SupportMessage>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListSupportsRequest {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListSupportsResponse {
    pub tg_handles: Vec<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListUsersRequest {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListUsersResponse {
    pub data: Vec<UserView>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoveSupportsRequest {
    pub tg_handles: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoveSupportsResponse {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendMsgRequest {
    pub message: String,
    pub server: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendMsgResponse {
    pub serverResponse: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetRoleRequest {
    pub user_public_id: Uuid,
    pub role: UserRole,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetRoleResponse {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubMsgEventsRequest {
    #[serde(default)]
    pub unsub: Option<bool>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubMsgEventsResponse {
    pub data: Vec<SupportMessage>,
}

impl WsRequest for InitRequest {
    type Response = InitResponse;
    const METHOD_ID: u32 = 11000;
    const ROLES: &[u32] = &[0];
    const SCHEMA: &'static str = r#"{
  "name": "Init",
  "code": 11000,
  "parameters": [
    {
      "name": "access_token",
      "ty": "String"
    }
  ],
  "returns": [
    {
      "name": "user_public_id",
      "ty": "UUID"
    },
    {
      "name": "role",
      "ty": {
        "EnumRef": {
          "name": "UserRole"
        }
      }
    }
  ],
  "stream_response": null,
  "description": "WIP",
  "json_schema": null,
  "roles": [
    "UserRole::Public"
  ]
}"#;
}
impl WsResponse for InitResponse {
    type Request = InitRequest;
}

impl WsRequest for SetRoleRequest {
    type Response = SetRoleResponse;
    const METHOD_ID: u32 = 21000;
    const ROLES: &[u32] = &[2];
    const SCHEMA: &'static str = r#"{
  "name": "SetRole",
  "code": 21000,
  "parameters": [
    {
      "name": "user_public_id",
      "ty": "UUID"
    },
    {
      "name": "role",
      "ty": {
        "EnumRef": {
          "name": "UserRole"
        }
      }
    }
  ],
  "returns": [],
  "stream_response": null,
  "description": "",
  "json_schema": null,
  "roles": [
    "UserRole::Admin"
  ]
}"#;
}
impl WsResponse for SetRoleResponse {
    type Request = SetRoleRequest;
}

impl WsRequest for ListUsersRequest {
    type Response = ListUsersResponse;
    const METHOD_ID: u32 = 21001;
    const ROLES: &[u32] = &[2];
    const SCHEMA: &'static str = r#"{
  "name": "ListUsers",
  "code": 21001,
  "parameters": [],
  "returns": [
    {
      "name": "data",
      "ty": {
        "StructTable": {
          "struct_ref": "UserView"
        }
      }
    }
  ],
  "stream_response": null,
  "description": "",
  "json_schema": null,
  "roles": [
    "UserRole::Admin"
  ]
}"#;
}
impl WsResponse for ListUsersResponse {
    type Request = ListUsersRequest;
}

impl WsRequest for AddSupportsRequest {
    type Response = AddSupportsResponse;
    const METHOD_ID: u32 = 21002;
    const ROLES: &[u32] = &[2];
    const SCHEMA: &'static str = r#"{
  "name": "AddSupports",
  "code": 21002,
  "parameters": [
    {
      "name": "tg_handles",
      "ty": "String"
    }
  ],
  "returns": [],
  "stream_response": null,
  "description": "",
  "json_schema": null,
  "roles": [
    "UserRole::Admin"
  ]
}"#;
}
impl WsResponse for AddSupportsResponse {
    type Request = AddSupportsRequest;
}

impl WsRequest for RemoveSupportsRequest {
    type Response = RemoveSupportsResponse;
    const METHOD_ID: u32 = 21003;
    const ROLES: &[u32] = &[2];
    const SCHEMA: &'static str = r#"{
  "name": "RemoveSupports",
  "code": 21003,
  "parameters": [
    {
      "name": "tg_handles",
      "ty": "String"
    }
  ],
  "returns": [],
  "stream_response": null,
  "description": "",
  "json_schema": null,
  "roles": [
    "UserRole::Admin"
  ]
}"#;
}
impl WsResponse for RemoveSupportsResponse {
    type Request = RemoveSupportsRequest;
}

impl WsRequest for ListSupportsRequest {
    type Response = ListSupportsResponse;
    const METHOD_ID: u32 = 21004;
    const ROLES: &[u32] = &[2];
    const SCHEMA: &'static str = r#"{
  "name": "ListSupports",
  "code": 21004,
  "parameters": [],
  "returns": [
    {
      "name": "tg_handles",
      "ty": {
        "Vec": "String"
      }
    }
  ],
  "stream_response": null,
  "description": "",
  "json_schema": null,
  "roles": [
    "UserRole::Admin"
  ]
}"#;
}
impl WsResponse for ListSupportsResponse {
    type Request = ListSupportsRequest;
}

impl WsRequest for SendMsgRequest {
    type Response = SendMsgResponse;
    const METHOD_ID: u32 = 31000;
    const ROLES: &[u32] = &[0, 1, 2];
    const SCHEMA: &'static str = r#"{
  "name": "SendMsg",
  "code": 31000,
  "parameters": [
    {
      "name": "message",
      "ty": "String"
    },
    {
      "name": "server",
      "ty": "String"
    }
  ],
  "returns": [
    {
      "name": "serverResponse",
      "ty": "String"
    }
  ],
  "stream_response": null,
  "description": "",
  "json_schema": null,
  "roles": [
    "UserRole::Admin",
    "UserRole::Authorized",
    "UserRole::Public"
  ]
}"#;
}
impl WsResponse for SendMsgResponse {
    type Request = SendMsgRequest;
}

impl WsRequest for ListMsgsRequest {
    type Response = ListMsgsResponse;
    const METHOD_ID: u32 = 31001;
    const ROLES: &[u32] = &[0, 1, 2];
    const SCHEMA: &'static str = r#"{
  "name": "ListMsgs",
  "code": 31001,
  "parameters": [],
  "returns": [
    {
      "name": "data",
      "ty": {
        "StructTable": {
          "struct_ref": "SupportMessage"
        }
      }
    }
  ],
  "stream_response": null,
  "description": "",
  "json_schema": null,
  "roles": [
    "UserRole::Admin",
    "UserRole::Authorized",
    "UserRole::Public"
  ]
}"#;
}
impl WsResponse for ListMsgsResponse {
    type Request = ListMsgsRequest;
}

impl WsRequest for SubMsgEventsRequest {
    type Response = SubMsgEventsResponse;
    const METHOD_ID: u32 = 31002;
    const ROLES: &[u32] = &[0, 1, 2];
    const SCHEMA: &'static str = r#"{
  "name": "SubMsgEvents",
  "code": 31002,
  "parameters": [
    {
      "name": "unsub",
      "ty": {
        "Optional": "Boolean"
      }
    }
  ],
  "returns": [
    {
      "name": "data",
      "ty": {
        "StructTable": {
          "struct_ref": "SupportMessage"
        }
      }
    }
  ],
  "stream_response": {
    "StructTable": {
      "struct_ref": "SupportMessage"
    }
  },
  "description": "",
  "json_schema": null,
  "roles": [
    "UserRole::Admin",
    "UserRole::Authorized",
    "UserRole::Public"
  ]
}"#;
}
impl WsResponse for SubMsgEventsResponse {
    type Request = SubMsgEventsRequest;
}

impl WsRequest for ListLeadsRequest {
    type Response = ListLeadsResponse;
    const METHOD_ID: u32 = 41000;
    const ROLES: &[u32] = &[2];
    const SCHEMA: &'static str = r#"{
  "name": "ListLeads",
  "code": 41000,
  "parameters": [],
  "returns": [
    {
      "name": "data",
      "ty": {
        "StructTable": {
          "struct_ref": "WaitlistLead"
        }
      }
    }
  ],
  "stream_response": null,
  "description": "Lists leads in the waitlist.",
  "json_schema": null,
  "roles": [
    "UserRole::Admin"
  ]
}"#;
}
impl WsResponse for ListLeadsResponse {
    type Request = ListLeadsRequest;
}

impl WsRequest for AddLeadRequest {
    type Response = AddLeadResponse;
    const METHOD_ID: u32 = 41001;
    const ROLES: &[u32] = &[0];
    const SCHEMA: &'static str = r#"{
  "name": "AddLead",
  "code": 41001,
  "parameters": [
    {
      "name": "name",
      "ty": "String"
    },
    {
      "name": "telegram",
      "ty": {
        "Optional": "String"
      }
    },
    {
      "name": "whatsApp",
      "ty": {
        "Optional": "String"
      }
    },
    {
      "name": "description",
      "ty": "String"
    }
  ],
  "returns": [],
  "stream_response": null,
  "description": "Adds a lead to the waitlist.",
  "json_schema": null,
  "roles": [
    "UserRole::Public"
  ]
}"#;
}
impl WsResponse for AddLeadResponse {
    type Request = AddLeadRequest;
}
