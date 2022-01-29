use std::collections::HashMap;
use std::fs;
use tracing::{info, instrument};

use anyhow::Result;
use itertools::Itertools;
use serde::Deserialize;

/// Database config: schema to DbCfg
#[derive(Debug)]
pub struct DbsCfg {
    pub dbs: HashMap<String, DbCfg>,
}

#[derive(Deserialize, Debug)]
struct DbCfgVec {
    databases: Vec<DbCfg>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DbCfg {
    pub schema: String,
    pub secret: String,
}

impl DbsCfg {
    #[instrument(name = "dbs_cfg_new", skip_all)]
    pub fn new(cfg_file: &str) -> Result<Self> {
        let content = fs::read_to_string(cfg_file)?;
        let db_cfg_vec: DbCfgVec = toml::from_str(&content)?;

        let res = db_cfg_vec
            .databases
            .into_iter()
            .into_group_map_by(|db| db.schema.clone())
            .into_iter()
            .map(|(schema, dbs)| (schema, dbs.first().unwrap().clone()))
            .collect::<HashMap<String, DbCfg>>();

        info!("Dbs cfg created");

        Ok(DbsCfg { dbs: res })
    }
}

#[cfg(test)]
mod cfg_test {
    use super::*;

    #[test]
    fn deserialize_cfg_success() {
        let cfg_file = "./tests/cfg.toml";
        let cfg = DbsCfg::new(cfg_file).unwrap();
        assert_eq!(cfg.dbs.get("db1").unwrap().schema, "db1");
    }
}
