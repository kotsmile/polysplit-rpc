use std::sync::Arc;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use rocket::async_trait;
use uuid::Uuid;

use crate::models::{Group, NewRpc, Rpc};

pub struct GroupService {
    group_storage: Arc<dyn GroupStorage>,
    group_cache: Arc<dyn GroupCache>,
}

#[async_trait]
pub trait GroupStorage: Send + Sync + 'static {
    async fn create_group(&self, new_group: &Group) -> Result<Option<Group>>;
    async fn update_api_key(&self, group_id: &Uuid, api_key: &str) -> Result<()>;
    async fn get_groups_for_user(&self, user_id: &Uuid) -> Result<Vec<Group>>;
    async fn get_groups(&self) -> Result<Vec<Group>>;
    async fn get_group_by_id(&self, group_id: &Uuid) -> Result<Option<Group>>;
    async fn get_group_rpcs(&self, group_id: &Uuid) -> Result<Vec<Rpc>>;
    async fn get_rpc_by_url(&self, url: &str) -> Result<Option<Rpc>>;
    async fn add_group_rpc(&self, group_id: &Uuid, rpc_id: &i32) -> Result<()>;
    async fn create_and_add_rpc_to_group(&self, group_id: &Uuid, new_rpc: &NewRpc) -> Result<Rpc>;
}

#[async_trait]
pub trait GroupCache: Send + Sync + 'static {
    async fn update_api_key(&self, old_api_key: &str, new_api_key: &str);
}

impl GroupService {
    pub fn new(group_cache: Arc<dyn GroupCache>, group_storage: Arc<dyn GroupStorage>) -> Self {
        Self {
            group_cache,
            group_storage,
        }
    }

    pub async fn create_group(&self, user_id: &Uuid, name: &str) -> Result<Option<Group>> {
        self.group_storage
            .create_group(&Group {
                id: Uuid::new_v4(),
                owner_id: user_id.clone(),
                name: name.to_string(),
                api_key: Uuid::new_v4().to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
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
        self.group_storage
            .update_api_key(group_id, &api_key)
            .await
            .context("failed to update api key in storage repo")?;

        self.group_cache
            .update_api_key(&group.api_key, &api_key)
            .await;
        Ok(api_key)
    }

    pub async fn get_groups_for_user(&self, user_id: &Uuid) -> Result<Vec<Group>> {
        self.group_storage
            .get_groups_for_user(user_id)
            .await
            .context("failed to find groups for user in storage repo")
    }

    pub async fn get_groups(&self) -> Result<Vec<Group>> {
        self.group_storage
            .get_groups()
            .await
            .context("failed to find groups in storage repo")
    }

    pub async fn get_group_by_id(&self, group_id: &Uuid) -> Result<Option<Group>> {
        self.group_storage
            .get_group_by_id(group_id)
            .await
            .context("failed to find group by id in storage repo")
    }

    pub async fn get_group_rpcs(&self, group_id: &Uuid) -> Result<Vec<Rpc>> {
        self.group_storage
            .get_group_rpcs(group_id)
            .await
            .context("failed to request rpcs for group in storage repo")
    }

    pub async fn add_rpc_to_group(&self, group_id: &Uuid, new_rpc: &NewRpc) -> Result<Rpc> {
        match self
            .group_storage
            .get_rpc_by_url(&new_rpc.url)
            .await
            .context("failed to request rpc in storage repo")?
        {
            Some(rpc) => self
                .group_storage
                .add_group_rpc(group_id, &rpc.id)
                .await
                .context("failed to add rpc to group in storage repo")
                .map(|_| rpc),
            None => self
                .group_storage
                .create_and_add_rpc_to_group(group_id, new_rpc)
                .await
                .context("failed to create rpc and add it to group in storage repo"),
        }
    }
}
