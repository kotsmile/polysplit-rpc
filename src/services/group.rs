use std::sync::Arc;

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::{
    models::{Group, NewGroup, NewUser, Rpc, User},
    repo::storage::StorageRepo,
};

pub struct GroupService {
    storage_repo: Arc<StorageRepo>,
}

impl GroupService {
    pub fn new(storage_repo: Arc<StorageRepo>) -> Self {
        Self { storage_repo }
    }

    pub async fn create_group(&self, user_id: &Uuid, name: &str) -> Result<Option<Group>> {
        self.storage_repo
            .create_group(&Group {
                id: Uuid::new_v4(),
                owner_id: user_id.clone(),
                name: name.to_string(),
                api_key: "".to_string(),
            })
            .await
            .context("failed to create new group")
    }

    pub async fn get_groups(&self, user_id: &Uuid) -> Result<Vec<Group>> {
        self.storage_repo
            .get_groups_for_user(user_id)
            .await
            .context("failed to find groups for user")
    }

    pub async fn get_group_by_id(&self, group_id: &Uuid) -> Result<Option<Group>> {
        self.storage_repo
            .get_group_by_id(group_id)
            .await
            .context("failed to find group by id")
    }

    // pub async fn get_groups_rpc(&self, group_id: &Uuid) -> Result<Vec<Rpc>> {}
}
