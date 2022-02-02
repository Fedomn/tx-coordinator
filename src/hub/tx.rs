use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use sqlx::{Pool, Postgres, Transaction};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

/// [The latter can be dynamically allocated at run-time, can be safely and freely mutated, can be dropped, and can live for arbitrary durations.](https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#2-if-t-static-then-t-must-be-valid-for-the-entire-program)
// #[async_trait]
// pub trait Tx: Send + Sync + 'static {
//     fn get_id(&self) -> String;
//     async fn execute(&self) -> Result<()>;
//     async fn commit(&self) -> Result<()>;
//     async fn rollback(&self) -> Result<()>;
// }

pub trait TxNew: Send + Sync + 'static {
    type TxR1<'a>: Future<Output = Result<()>>
    where
        Self: 'a;
    type TxR2<'a>: Future<Output = Result<()>>
    where
        Self: 'a;
    type TxR3<'a>: Future<Output = Result<()>>
    where
        Self: 'a;
    fn get_id(&self) -> String;
    fn execute(&self) -> Self::TxR1<'_>;
    fn commit(&self) -> Self::TxR2<'_>;
    fn rollback(&self) -> Self::TxR3<'_>;
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

// #[async_trait]
impl TxNew for CopyDataTx {
    type TxR1<'a>
    where
        Self: 'a,
    = impl Future<Output = Result<()>>;
    type TxR2<'a>
    where
        Self: 'a,
    = impl Future<Output = Result<()>>;
    type TxR3<'a>
    where
        Self: 'a,
    = impl Future<Output = Result<()>>;

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn execute(&self) -> Self::TxR1<'_> {
        async move {
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
    }

    fn commit(&self) -> Self::TxR2<'_> {
        async move {
            let mut mutex = self._raw_tx.lock().await;
            let tx = mutex.take().unwrap();
            tx.commit().await?;
            Ok(())
        }
    }

    fn rollback(&self) -> Self::TxR3<'_> {
        async move {
            let mut mutex = self._raw_tx.lock().await;
            let tx = mutex.take().unwrap();
            tx.rollback().await?;
            Ok(())
        }
    }
}
