use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::{Error, Pool, Postgres};

#[derive(Debug)]
pub struct DB {
    pub schema: String,
    pub secret: String,
    pub sql_files: Vec<String>,
}

impl DB {
    pub async fn gen_pool(&self) -> Result<Pool<Postgres>, Error> {
        PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(3))
            .connect(self.secret.as_str())
            .await
    }
}
