use std::sync::Arc;

use futures::future;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info, warn};

use crate::{CopyDataTx, TxNew};

pub struct TxCoordinator {
    // Vec<Box<dyn Tx>> refer: https://stackoverflow.com/questions/66901861/incorrect-type-inference-for-rust-vector-of-trait-object
    txs: Arc<Vec<Arc<CopyDataTx>>>,
}

impl TxCoordinator {
    pub fn new(txs: Vec<Arc<CopyDataTx>>) -> Self {
        Self { txs: Arc::new(txs) }
    }

    pub async fn commit_or_rollback(&self) -> anyhow::Result<()> {
        let (commit_send, commit_recv) = oneshot::channel();
        let (rollback_send, rollback_recv) = mpsc::unbounded_channel();
        let (done_send, mut done_recv) = mpsc::unbounded_channel();

        let _tx1 = self.txs.clone();
        let _rs1 = rollback_send.clone();
        tokio::spawn(async {
            execute(_tx1, commit_send, _rs1).await;
        });

        let _tx2 = self.txs.clone();
        let _ds1 = done_send.clone();
        let _rs2 = rollback_send.clone();
        tokio::spawn(async move {
            commit(_tx2, commit_recv, _ds1, _rs2).await;
        });

        let _tx3 = self.txs.clone();
        let _ds2 = done_send.clone();
        tokio::spawn(async move {
            rollback(_tx3, _ds2, rollback_recv).await;
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

async fn execute(
    txs: Arc<Vec<Arc<CopyDataTx>>>,
    commit_send: Sender<()>,
    rollback_send: UnboundedSender<()>,
) {
    let mut tasks = vec![];
    for tx in txs.iter() {
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
                rollback_send.send(()).unwrap()
            }
        }
        Err(res) => {
            debug!("exec -> rollback_send, {:?}", res);
            rollback_send.send(()).unwrap()
        }
    };
}

async fn commit(
    txs: Arc<Vec<Arc<CopyDataTx>>>,
    commit_recv: Receiver<()>,
    done_send: UnboundedSender<()>,
    rollback_send: UnboundedSender<()>,
) {
    match commit_recv.await {
        Ok(_) => {
            info!("prepare commit");
            let mut tasks = vec![];

            for tx in txs.iter() {
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
                Ok(_) => done_send.send(()).unwrap(),
                Err(_) => rollback_send.send(()).unwrap(),
            }
        }
        Err(_) => {
            info!("commit_send dropped, drop commit phase");
        }
    }
}

async fn rollback(
    txs: Arc<Vec<Arc<CopyDataTx>>>,
    done_send: UnboundedSender<()>,
    mut rollback_recv: UnboundedReceiver<()>,
) {
    while let Some(()) = rollback_recv.recv().await {
        info!("prepare rollback");
        let mut tasks = vec![];
        for tx in txs.iter() {
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
            done_send.closed().await;
        } else {
            info!("rollback all txs done");
            done_send.send(()).unwrap();
        }
    }
}
