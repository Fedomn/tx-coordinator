use std::sync::Arc;

use anyhow::Result;
use tracing::{info, Span};
use tracing_appender::non_blocking::WorkerGuard;

use crate::cfg::DbsCfg;
use crate::hub::tx::Tx;
use crate::hub::{coordinator, Hub};

mod cfg;
pub mod hub;
mod log;

pub fn init_log() -> (WorkerGuard, Span) {
    let (_file_guard, root) = log::init_log();
    (_file_guard, root)
}

pub fn read_cfg(cfg_file: &str, dir: &str) -> Result<Hub> {
    let dbs_cfg = DbsCfg::new(cfg_file)?;
    Ok(Hub::new(dir, &dbs_cfg))
}

pub async fn execute(txs: Vec<Arc<dyn Tx>>) -> Result<()> {
    let coordinator = coordinator::TxCoordinator::new(txs);
    match coordinator.commit_or_rollback().await {
        Ok(_) => {
            info!("Migration done.");
            Ok(())
        }
        Err(e) => {
            info!("Migration failed: {}", e);
            Err(e)
        }
    }
}
