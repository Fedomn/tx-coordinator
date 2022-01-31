use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use sqlx::{Pool, Postgres, Transaction};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

/// https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#2-if-t-static-then-t-must-be-valid-for-the-entire-program
/// The latter can be dynamically allocated at run-time, can be safely and freely mutated, can be dropped, and can live for arbitrary durations.
#[async_trait]
pub trait Tx: Send + Sync + 'static {
    fn get_id(&self) -> String;
    async fn execute(&self) -> Result<()>;
    async fn commit(&self) -> Result<()>;
    async fn rollback(&self) -> Result<()>;
}

pub struct CopyDataTx {
    id: String,
    // use option in mutex for Transaction to move ownership in commit/rollback method
    // refer: https://stackoverflow.com/questions/30573188/cannot-move-data-out-of-a-mutex
    _raw_tx: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,
    sql_files: Vec<String>,
}

impl CopyDataTx {
    pub async fn new(
        id: impl Into<String>,
        pool: Pool<Postgres>,
        sql_files: Vec<String>,
    ) -> Result<CopyDataTx> {
        let tx = pool.begin().await?;
        Ok(CopyDataTx {
            id: id.into(),
            _raw_tx: Arc::new(Mutex::new(Some(tx))),
            sql_files,
        })
    }
}

#[async_trait]
impl Tx for CopyDataTx {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    async fn execute(&self) -> Result<()> {
        let arc = self._raw_tx.clone();
        let mut mutex = arc.lock().await;
        let tx = mutex.as_mut().unwrap();

        let mut stream = tokio_stream::iter(&self.sql_files);
        while let Some(file) = stream.next().await {
            let sql = std::fs::read_to_string(file)?;
            sqlx::query(&sql).execute(&mut *tx).await?;
        }

        Ok(())
    }

    async fn commit(&self) -> Result<()> {
        let mut mutex = self._raw_tx.lock().await;
        let tx = mutex.take().unwrap();
        tx.commit().await?;
        Ok(())
    }

    async fn rollback(&self) -> Result<()> {
        let mut mutex = self._raw_tx.lock().await;
        let tx = mutex.take().unwrap();
        tx.rollback().await?;
        Ok(())
    }
}

#[cfg(test)]
mod copy_data_tx_test {
    use std::sync::Arc;

    use anyhow::Result;

    use crate::hub::db::DB;
    use crate::hub::tx::{CopyDataTx, Tx};

    async fn run(tx: Arc<dyn Tx>) -> Result<()> {
        // use &self to avoid: cannot move a value of type dyn Tx: the size of dyn Tx cannot be statically determined
        tx.execute().await?;
        tx.rollback().await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn tx_rollback_works() -> Result<()> {
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
}
