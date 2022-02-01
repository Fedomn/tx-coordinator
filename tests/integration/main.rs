#[cfg(test)]
mod integration_test {
    use std::sync::Arc;

    use anyhow::Result;

    use tx_coordinator::hub::db::DB;
    use tx_coordinator::hub::tx::{CopyDataTx, Tx};
    use tx_coordinator::{execute, init_log, read_cfg};

    #[tokio::test]
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

    #[tokio::test]
    async fn tx_rollback_works() -> Result<()> {
        async fn run(tx: Arc<dyn Tx>) -> Result<()> {
            // use &self to avoid: cannot move a value of type dyn Tx: the size of dyn Tx cannot be statically determined
            tx.execute().await?;
            tx.rollback().await?;
            Ok(())
        }

        let db = DB {
            schema: "db1".to_string(),
            secret: "postgres://postgres:@localhost/db1".to_string(),
            sql_files: vec!["./tests/sqlfiles/0-db1-test.sql".to_string()],
        };
        let pool = db.gen_pool().await?;

        let tx = CopyDataTx::new("id", pool, db.sql_files.clone()).await?;
        run(Arc::new(tx)).await?;

        // check that inserted value is now gone
        let pool2 = db.gen_pool().await?;
        let inserted_todo = sqlx::query(r#"SELECT FROM "db1"."test_table" WHERE id = 1"#)
            .fetch_one(&pool2)
            .await;

        assert!(inserted_todo.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn it_main_works() -> Result<()> {
        let (_file_guard, root) = init_log();
        let _enter = root.enter();

        let hub_res = read_cfg("./tests/cfg.toml", "./tests/sqlfiles")?;
        let txs = hub_res.build_tx().await?;
        let result = execute(txs).await;

        let db = DB {
            schema: "db1".to_string(),
            secret: "postgres://postgres:@localhost/db1".to_string(),
            sql_files: vec![],
        };

        // check rollback works
        let pool = db.gen_pool().await?;
        let inserted_todo = sqlx::query(r#"SELECT FROM "db1"."test_table" WHERE id = 1"#)
            .fetch_one(&pool)
            .await;
        assert!(inserted_todo.is_err());

        result
    }
}
