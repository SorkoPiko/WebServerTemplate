use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use crate::model::database::Database;

pub struct PostgresDatabase {
    pool: sqlx::PgPool,
}

impl PostgresDatabase {
    pub async fn new(connection_string: &str, max_connections: u32) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(connection_string)
            .await
            .context("Failed to connect to PostgreSQL")?;

        // sqlx::migrate!("./migrations")
        //     .run(&pool)
        //     .await
        //     .context("Failed to run database migrations")?;

        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl Database for PostgresDatabase {}