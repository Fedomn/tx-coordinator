use sqlx::postgres::PgPoolOptions;
use sqlx::{Error, Pool, Postgres};

pub struct DB {
    pub schema: String,
    pub secret: String,
    pub sql_files: Vec<String>,
}

impl DB {
    pub async fn gen_pool(&self) -> Result<Pool<Postgres>, Error> {
        PgPoolOptions::new()
            .max_connections(1)
            .connect(self.secret.as_str())
            .await
    }
}

#[cfg(test)]
mod db_test {
    use anyhow::Result;

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn connect_local_db_works() -> Result<()> {
        let db = DB {
            schema: "db1".to_string(),
            secret: "postgres://postgres:@localhost/db1".to_string(),
            sql_files: vec![],
        };

        let pool = db.gen_pool().await?;

        let row: (i64,) = sqlx::query_as("SELECT $1")
            .bind(150_i64)
            .fetch_one(&pool)
            .await?;

        assert_eq!(row.0, 150);

        Ok(())
    }
}
