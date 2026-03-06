use async_trait::async_trait;
use eyre::{Context, ContextCompat};
use honey_id_types::handlers::convenience_utils::user_management::{CreateUserInfo, UserStorage};
use uuid::Uuid;
use worktable::{prelude::*, worktable};

use crate::codegen::model::UserRole;

worktable!(
    name: User,
    persist: true,
    columns: {
        id: i64 primary_key autoincrement,
        pub_id: Uuid,
        username: String,
        role: UserRole,
    },
    indexes: {
        pub_id_idx: pub_id unique,
        name_idx: username unique,
    },
    queries: {
        update: {
            RoleById(role) by id,
        }
    }
);

#[async_trait]
impl UserStorage for UserWorkTable {
    fn get_api_roles_by_pub_id(
        &self,
        user_pub_id: honey_id_types::id_entities::UserPublicId,
    ) -> eyre::Result<Vec<u32>> {
        Ok(vec![
            self.select_by_pub_id(Uuid::from(user_pub_id))
                .wrap_err("User not found")?
                .role as u32,
        ])
    }

    fn get_public_roles(&self) -> &[u32] {
        &[UserRole::Public as _]
    }

    fn get_honey_auth_role(&self) -> u32 {
        UserRole::HoneyAuth as _
    }

    async fn create_or_update_user(&self, user_info_request: CreateUserInfo) -> eyre::Result<()> {
        let mut user = self.select_by_pub_id(user_info_request.user_pub_id);

        if let Some(user) = &mut user {
            user.username = user_info_request.username;

            let _ = self.update(user.to_owned()).await.wrap_err("Error updating user")?;

            Ok(())
        } else {
            let user_role = UserRole::Authorized;

            let _user_id = self
                .insert(UserRow {
                    id: self.get_next_pk().into(),
                    pub_id: user_info_request.user_pub_id,
                    username: user_info_request.username.clone(),
                    role: user_role,
                })
                .wrap_err("Error inserting new user")?;

            Ok(())
        }
    }
}
