use std::sync::Arc;

use futures::future;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info, warn};

use crate::hub::tx::Tx;

pub struct TxCoordinator {
    // Vec<Box<dyn Tx>> refer: https://stackoverflow.com/questions/66901861/incorrect-type-inference-for-rust-vector-of-trait-object
    txs: Arc<Vec<Arc<dyn Tx>>>,
}

impl TxCoordinator {
    pub fn new(txs: Vec<Arc<dyn Tx>>) -> Self {
        Self { txs: Arc::new(txs) }
    }

    pub async fn commit_or_rollback(&self) -> anyhow::Result<()> {
        let (commit_send, commit_recv) = oneshot::channel();
        let (rollback_send, mut rollback_recv) = mpsc::unbounded_channel();
        let (done_send, mut done_recv) = mpsc::unbounded_channel();

        let _tx1 = self.txs.clone();
        let _rs1 = rollback_send.clone();

        tokio::spawn(async move {
            let mut tasks = vec![];
            for tx in _tx1.iter() {
                let _tx = tx.clone();
                let handle = tokio::spawn(async move {
                    let tx_id = _tx.get_id();
                    match _tx.execute().await {
                        Ok(_) => {
                            info!("exec tx[{}] succeed", tx_id);
                            Ok(tx_id)
                        }
                        Err(e) => {
                            info!("exec tx[{}] fails with err: [{}]", tx_id, e);
                            Err(tx_id)
                        }
                    }
                });
                tasks.push(handle);
            }
            match future::try_join_all(tasks).await {
                Ok(res) => {
                    let errs = res.into_iter().filter(|x| x.is_err()).collect::<Vec<_>>();
                    if errs.is_empty() {
                        debug!("exec -> commit_send");
                        commit_send.send(()).unwrap()
                    } else {
                        debug!("exec -> rollback_send, {:?}", errs);
                        _rs1.send(()).unwrap()
                    }
                }
                Err(res) => {
                    debug!("exec -> rollback_send, {:?}", res);
                    _rs1.send(()).unwrap()
                }
            };
        });

        let _tx2 = self.txs.clone();
        let _ds1 = done_send.clone();
        let _rs2 = rollback_send.clone();
        tokio::spawn(async move {
            match commit_recv.await {
                Ok(_) => {
                    info!("prepare commit");
                    let mut tasks = vec![];

                    for tx in _tx2.iter() {
                        let _tx = tx.clone();
                        let handle = tokio::spawn(async move {
                            let tx_id = _tx.get_id();
                            match _tx.commit().await {
                                Ok(_) => {
                                    info!("commit tx[{}] succeed", tx_id);
                                    Ok(tx_id)
                                }
                                Err(e) => {
                                    info!("commit tx[{}] fails with err: [{}]", tx_id, e);
                                    Err(tx_id)
                                }
                            }
                        });
                        tasks.push(handle);
                    }
                    match future::try_join_all(tasks).await {
                        Ok(_) => _ds1.send(()).unwrap(),
                        Err(_) => _rs2.send(()).unwrap(),
                    }
                }
                Err(_) => {
                    info!("commit_send dropped, drop commit phase");
                }
            }
        });

        let _tx3 = self.txs.clone();
        let _ds2 = done_send.clone();
        tokio::spawn(async move {
            while let Some(()) = rollback_recv.recv().await {
                info!("prepare rollback");
                let mut tasks = vec![];
                for tx in _tx3.iter() {
                    let _tx = tx.clone();
                    let handle = tokio::spawn(async move {
                        let tx_id = _tx.get_id();
                        match _tx.rollback().await {
                            Ok(_) => {
                                info!("rollback tx[{}] succeed", tx_id);
                                Ok(tx_id)
                            }
                            Err(e) => {
                                info!("rollback tx[{}] fails with err: [{}]", tx_id, e);
                                Err(tx_id)
                            }
                        }
                    });
                    tasks.push(handle);
                }
                let res = future::join_all(tasks).await;
                let errs = res.into_iter().filter(|x| x.is_err()).collect::<Vec<_>>();
                if !errs.is_empty() {
                    warn!("rollback encounter errs: {:?}", errs);
                    _ds2.closed().await;
                } else {
                    info!("rollback all txs done");
                    _ds2.send(()).unwrap();
                }
            }
        });

        return match done_recv.recv().await {
            Some(_) => {
                info!("done recv ok ");
                Ok(())
            }
            None => {
                info!("done recv none");
                Ok(())
            }
        };
    }
}
