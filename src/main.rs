use anyhow::Result;
use clap::Parser;
use tracing::info;

use cfg::DbsCfg;

use hub::Hub;

mod cfg;
mod hub;
mod log;

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
    let (_file_guard, root) = log::init_log();
    let _enter = root.enter();
    info!("Starting Tx-Coordinator");

    let args = Args::parse();
    info!("Got args: {:?}", args);

    let dbs_cfg = DbsCfg::new(&args.cfg).unwrap();
    let _dbs_hub = Hub::new(&args.dir, &dbs_cfg);

    Ok(())
}
