use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "Tx-Coordinator", author = "fedomn", version = "1.0.0")]
pub struct Args {
    #[clap(help = "The path of configuration file", default_value = "./cfg.toml")]
    cfg: String,
}

fn main() {
    let args = Args::parse();
    println!("Got args: {:?}", args);
}
