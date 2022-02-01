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
