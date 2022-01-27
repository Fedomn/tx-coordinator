use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "Tx-Coordinator", author = "fedomn", version = "1.0.0")]
pub struct Args {
    #[clap(help = "The path of configuration file", default_value = "./cfg.toml")]
    cfg: String,

    #[clap(help = "The directory of executed sql files", default_value = "./sqlfiles")]
    dir: String,
}

fn main() {
    let args = Args::parse();
    println!("Got args: {:?}", args);
}
