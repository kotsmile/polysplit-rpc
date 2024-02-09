use std::sync::Arc;

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::{
    models::{NewUser, User},
    repo::storage::StorageRepo,
};

pub struct UserService {
    storage_repo: Arc<StorageRepo>,
}

impl UserService {
    pub fn new(storage_repo: Arc<StorageRepo>) -> Self {
        Self { storage_repo }
    }

    pub async fn create_user(&self, new_user: &NewUser) -> Result<Option<User>> {
        self.storage_repo
            .create_user(&User {
                id: Uuid::new_v4(),
                email: new_user.email.clone(),
            })
            .await
            .context("failed to create new user in storage repo")
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        self.storage_repo
            .get_user_by_email(email)
            .await
            .context("failed to find user by email in storage repo")
    }
}
