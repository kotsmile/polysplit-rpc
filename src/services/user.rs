use std::sync::Arc;

use anyhow::{Context, Result};
use rocket::async_trait;
use uuid::Uuid;

use crate::models::{NewUser, User};

pub struct UserService {
    user_storage: Arc<dyn UserStorage>,
}

#[async_trait]
pub trait UserStorage: Send + Sync + 'static {
    async fn create_user(&self, new_user: &User) -> Result<Option<User>>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;
}

impl UserService {
    pub fn new(user_storage: Arc<dyn UserStorage>) -> Self {
        Self { user_storage }
    }

    pub async fn create_user(&self, new_user: &NewUser) -> Result<Option<User>> {
        self.user_storage
            .create_user(&User {
                id: Uuid::new_v4(),
                email: new_user.email.clone(),
            })
            .await
            .context("failed to create new user in storage repo")
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        self.user_storage
            .get_user_by_email(email)
            .await
            .context("failed to find user by email in storage repo")
    }
}
