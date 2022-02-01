use anyhow::Result;
use clap::Parser;
use tracing::info;

use txcoordinator::{execute, init_log, read_cfg};

#[derive(Parser, Debug)]
#[clap(name = "Tx-Coordinator", author = "fedomn", version = "1.0.0")]
pub struct Args {
    #[clap(
        long,
        help = "The path of configuration file",
        default_value = "./cfg.toml"
    )]
    cfg: String,

    #[clap(
        long,
        help = "The directory of executed sql files",
        default_value = "./sqlfiles"
    )]
    dir: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let (_file_guard, root) = init_log();
    let _enter = root.enter();
    info!("Starting Tx-Coordinator");

    let args = Args::parse();
    info!("Got args: {:?}", args);

    let dbs_hub = read_cfg(&args.cfg, &args.dir)?;
    info!("Read cfg: {:?}", dbs_hub);

    let txs = dbs_hub.build_tx().await?;
    info!("Init transaction done");

    match execute(txs).await {
        Ok(_) => {
            info!("Migration done");
            Ok(())
        }
        Err(e) => {
            info!("Migration failed: {}", e);
            Err(e)
        }
    }
}
