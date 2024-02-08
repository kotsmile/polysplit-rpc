use anyhow::{bail, Context, Result};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use uuid::Uuid;

use crate::models::{Chain, Group, NewRpc, Rpc, User};

pub struct StorageRepo {
    pool: Pool<Postgres>,
}

impl StorageRepo {
    pub async fn new(database_url: String, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(&database_url)
            .await
            .context("failed to initiate connection")?;

        sqlx::migrate!()
            .run(&pool)
            .await
            .context("failed to run migrations")?;

        Ok(Self { pool })
    }

    pub async fn create_user(&self, new_user: &User) -> Result<Option<User>> {
        sqlx::query_as!(
            User,
            "insert into users (id, email) values ($1, $2) returning *;",
            new_user.id,
            new_user.email,
        )
        .fetch_optional(&self.pool)
        .await
        .context("failed to insert row in users table")
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        sqlx::query_as!(User, "select * from users where email = $1;", email)
            .fetch_optional(&self.pool)
            .await
            .context("failed to select user with email")
    }

    pub async fn get_chains(&self) -> Result<Vec<Chain>> {
        sqlx::query_as!(Chain, "select * from chains;")
            .fetch_all(&self.pool)
            .await
            .context("failed to select chains in table")
    }

    pub async fn create_chain(&self, new_chain: &Chain) -> Result<Option<Chain>> {
        sqlx::query_as!(
            Chain,
            "insert into chains (id, name) values ($1, $2) returning *;",
            new_chain.id,
            new_chain.name
        )
        .fetch_optional(&self.pool)
        .await
        .context("failed to insert row in chains table")
    }

    pub async fn get_rpcs(&self) -> Result<Vec<Rpc>> {
        sqlx::query_as!(Rpc, "select * from rpcs;")
            .fetch_all(&self.pool)
            .await
            .context("failed to select from rpcs")
    }

    pub async fn get_rpcs_by_chain_id(&self, chain_id: &str) -> Result<Vec<Rpc>> {
        sqlx::query_as!(Rpc, "select * from rpcs where chain_id = $1;", chain_id)
            .fetch_all(&self.pool)
            .await
            .context("failed to select from rpcs")
    }

    pub async fn create_rpc(&self, new_rpc: &NewRpc) -> Result<Option<Rpc>> {
        sqlx::query_as!(
            Rpc,
            "insert into rpcs (chain_id, url) values ($1, $2) returning *;",
            new_rpc.chain_id,
            new_rpc.url
        )
        .fetch_optional(&self.pool)
        .await
        .context("failed to insert row in rpcs table")
    }

    pub async fn get_groups(&self) -> Result<Vec<Group>> {
        sqlx::query_as!(Group, "select * from groups")
            .fetch_all(&self.pool)
            .await
            .context("failed to get all groups")
    }

    pub async fn create_group(&self, new_group: &Group) -> Result<Option<Group>> {
        sqlx::query_as!(
            Group,
            "insert into groups (id, name, owner_id, api_key) values ($1, $2, $3, $4) returning *;",
            new_group.id,
            new_group.name,
            new_group.owner_id,
            new_group.api_key,
        )
        .fetch_optional(&self.pool)
        .await
        .context("failed to insert row in groups table")
    }

    pub async fn get_groups_for_user(&self, user_id: &Uuid) -> Result<Vec<Group>> {
        sqlx::query_as!(Group, "select * from groups where owner_id = $1", user_id)
            .fetch_all(&self.pool)
            .await
            .context("failed to find groups for user")
    }

    pub async fn get_group_by_id(&self, group_id: &Uuid) -> Result<Option<Group>> {
        sqlx::query_as!(Group, "select * from groups where id = $1", group_id)
            .fetch_optional(&self.pool)
            .await
            .context("failed to find group")
    }

    pub async fn get_group_rpcs(&self, group_id: &Uuid) -> Result<Vec<Rpc>> {
        sqlx::query_as!(
            Rpc,
            r#"
                select r.id, r.chain_id, r.url 
                from rpcs r 
                left join groups_rpcs 
                on groups_rpcs.rpc_id = r.id 
                where groups_rpcs.group_id = $1
            "#,
            group_id
        )
        .fetch_all(&self.pool)
        .await
        .context("failed to get rpcs for group")
    }

    pub async fn get_rpc_by_url(&self, url: &str) -> Result<Option<Rpc>> {
        sqlx::query_as!(Rpc, "select * from rpcs where url = $1", url)
            .fetch_optional(&self.pool)
            .await
            .context("failed to find rpc")
    }

    pub async fn create_and_add_rpc_to_group(
        &self,
        group_id: &Uuid,
        new_rpc: &NewRpc,
    ) -> Result<Rpc> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("failed to init transaction")?;

        let rpc = sqlx::query_as!(
            Rpc,
            "insert into rpcs(chain_id, url) values ($1, $2) returning *;",
            new_rpc.chain_id,
            new_rpc.url
        )
        .fetch_optional(&mut *tx)
        .await
        .context("failed to insert new rpc")?;

        let Some(rpc) = rpc else {
            bail!("failed to retrieve rpc after inserting it")
        };

        sqlx::query!(
            "insert into groups_rpcs(group_id,rpc_id) values ($1, $2)",
            group_id,
            rpc.id,
        )
        .execute(&mut *tx)
        .await
        .context("failed to insert new group rpc pair")?;

        tx.commit()
            .await
            .context("failed to finalize transaction")?;

        Ok(rpc)
    }

    pub async fn add_group_rpc(&self, group_id: &Uuid, rpc_id: &i32) -> Result<()> {
        sqlx::query!(
            "insert into groups_rpcs(group_id, rpc_id) values ($1, $2)",
            group_id,
            rpc_id
        )
        .execute(&self.pool)
        .await
        .context("failed to insert rpc group record")
        .map(|_| {})
    }

    // pub async fn get_group_rpcs(&self, group_id:&Uuid) -> Result<>
}
