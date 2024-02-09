use std::sync::Arc;

use anyhow::{bail, Context, Result};
use rocket::tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    models::{Group, NewRpc, Rpc},
    repo::{cache::CacheRepo, storage::StorageRepo},
};

pub struct GroupService {
    storage_repo: Arc<StorageRepo>,
    cache_repo: Arc<RwLock<CacheRepo>>,
}

impl GroupService {
    pub fn new(storage_repo: Arc<StorageRepo>, cache_repo: Arc<RwLock<CacheRepo>>) -> Self {
        Self {
            storage_repo,
            cache_repo,
        }
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
            .context("failed to create new group in storage repo")
    }

    // TODO: remake on sql
    pub async fn get_group_with_owner(&self, user_id: &Uuid, group_id: &Uuid) -> Result<Group> {
        let group = self
            .get_group_by_id(&group_id)
            .await
            .context("failed to find group by id in storage repo")?;
        let Some(group) = group else {
            bail!("no group was found for: {group_id}")
        };

        if &group.owner_id != user_id {
            bail!("user {user_id} is not owner of group {group_id}")
        }

        Ok(group)
    }

    pub async fn update_api_key(&self, group_id: &Uuid) -> Result<String> {
        let group = self
            .get_group_by_id(group_id)
            .await
            .context("failed to find group by id in storage repo")?;
        let Some(group) = group else {
            bail!("no group with given id")
        };

        let api_key = Uuid::new_v4().to_string();
        self.storage_repo
            .update_api_key(group_id, &api_key)
            .await
            .context("failed to update api key in storage repo")?;

        self.cache_repo
            .write()
            .await
            .update_api_key(&group.api_key, &api_key);
        Ok(api_key)
    }

    pub async fn get_groups_for_user(&self, user_id: &Uuid) -> Result<Vec<Group>> {
        self.storage_repo
            .get_groups_for_user(user_id)
            .await
            .context("failed to find groups for user in storage repo")
    }

    pub async fn get_groups(&self) -> Result<Vec<Group>> {
        self.storage_repo
            .get_groups()
            .await
            .context("failed to find groups in storage repo")
    }

    pub async fn get_group_by_id(&self, group_id: &Uuid) -> Result<Option<Group>> {
        self.storage_repo
            .get_group_by_id(group_id)
            .await
            .context("failed to find group by id in storage repo")
    }

    pub async fn get_group_rpcs(&self, group_id: &Uuid) -> Result<Vec<Rpc>> {
        self.storage_repo
            .get_group_rpcs(group_id)
            .await
            .context("failed to request rpcs for group in storage repo")
    }

    pub async fn add_rpc_to_group(&self, group_id: &Uuid, new_rpc: &NewRpc) -> Result<Rpc> {
        match self
            .storage_repo
            .get_rpc_by_url(&new_rpc.url)
            .await
            .context("failed to request rpc in storage repo")?
        {
            Some(rpc) => self
                .storage_repo
                .add_group_rpc(group_id, &rpc.id)
                .await
                .context("failed to add rpc to group in storage repo")
                .map(|_| rpc),
            None => self
                .storage_repo
                .create_and_add_rpc_to_group(group_id, new_rpc)
                .await
                .context("failed to create rpc and add it to group in storage repo"),
        }
    }
}
