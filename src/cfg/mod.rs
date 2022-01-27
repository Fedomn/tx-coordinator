use std::fs;

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Cfg {
    databases: Vec<DbCfg>,
}

#[derive(Deserialize, Debug)]
pub struct DbCfg {
    schema: String,
    secret: String,
}

impl Cfg {
    pub fn new(cfg_file: &str) -> Result<Cfg> {
        let content = fs::read_to_string(cfg_file)?;
        let result: Cfg = toml::from_str(&content)?;
        Ok(result)
    }
}

#[cfg(test)]
mod cfg_test {
    use super::*;

    #[test]
    fn deserialize_cfg_success() {
        let cfg_file = "./tests/cfg.toml";
        let cfg = Cfg::new(cfg_file).unwrap();
        assert_eq!(cfg.databases.len(), 3);
        assert_eq!(cfg.databases[0].schema, "db1");
    }
}
