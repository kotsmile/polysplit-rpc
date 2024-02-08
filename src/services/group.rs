use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use uuid::Uuid;

use crate::{
    models::{Group, NewRpc, Rpc},
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
                api_key: Uuid::new_v4().to_string(),
            })
            .await
            .context("failed to create new group")
    }

    // TODO remake on sql
    pub async fn get_group_with_owner(&self, user_id: &Uuid, group_id: &Uuid) -> Result<Group> {
        let group = self
            .get_group_by_id(&group_id)
            .await
            .context("failed to find group by id")?;
        let Some(group) = group else {
            bail!("no group was found for: {group_id}")
        };

        if &group.owner_id != user_id {
            bail!("user {user_id} is not owner of group {group_id}")
        }

        Ok(group)
    }

    pub async fn update_api_key(&self, group_id: &Uuid) -> Result<String> {
        self.get_group_by_id(group_id)
            .await
            .context("failed to group")
            .and_then(|v| v.ok_or(anyhow!("no group with given group id")))
            .map(|v| v.api_key)
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

    pub async fn get_group_rpcs(&self, group_id: &Uuid) -> Result<Vec<Rpc>> {
        self.storage_repo
            .get_group_rpcs(group_id)
            .await
            .context("failed to request rpcs for group")
    }

    pub async fn add_rpc_to_group(&self, group_id: &Uuid, new_rpc: &NewRpc) -> Result<Rpc> {
        let rpc = self
            .storage_repo
            .get_rpc_by_url(&new_rpc.url)
            .await
            .context("failed to request rpc")?;

        match rpc {
            Some(rpc) => self
                .storage_repo
                .add_group_rpc(group_id, &rpc.id)
                .await
                .context("failed to add rpc to group")
                .map(|_| rpc),
            None => self
                .storage_repo
                .create_and_add_rpc_to_group(group_id, new_rpc)
                .await
                .context("failed to create rpc and add it to group"),
        }
    }
}
