
# API Reference

## Structs/Datamodels

```rust
struct SupportMessage{ incoming: bool, sentBy: String, sentAt: String, content: String }


struct UserView{ userPubId: Uuid, username: String, role: UserRole }

```
---

## Enums

```rust
enum UserRole { Public, Authorized, Admin, HoneyAuth }

```
---

        

## auth Server
ID: 1
### Endpoints
|Code|Name|Parameters|Response|Description|FE Facing|
|-----------|-----------|----------|--------|-----------|-----------|
|11000|Init|`accessToken: String`|`userId: Uuid`, `role: UserRole`|WIP|true|

## admin Server
ID: 2
### Endpoints
|Code|Name|Parameters|Response|Description|FE Facing|
|-----------|-----------|----------|--------|-----------|-----------|
|21000|SetRole|`userPubId: Uuid`, `role: UserRole`|||true|
|21001|ListUsers||`data: Vec<UserView>`||true|
|21002|AddSupports|`tgHandles: String`|||true|
|21003|RemoveSupports|`tgHandles: String`|||true|
|21004|ListSupports||`tgHandles: Vec<String>`||true|

## support Server
ID: 3
### Endpoints
|Code|Name|Parameters|Response|Description|FE Facing|
|-----------|-----------|----------|--------|-----------|-----------|
|31000|SendMsg|`message: String`|||true|
|31001|ListMsgs||`data: Vec<SupportMessage>`||true|
|31002|SubMsgEvents|`unsub: Option<bool>`|`data: Vec<SupportMessage>`||true|
