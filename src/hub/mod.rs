use std::collections::HashMap;

use itertools::Itertools;
use tracing::{info, instrument};

use crate::cfg::DbsCfg;
use crate::hub::db::DB;

pub mod db;
pub mod tx;

/// Hub organizes database schema and sql files.
pub struct Hub {
    dbs: HashMap<String, DB>,
}

impl Hub {
    #[instrument(name = "hub_new", skip_all)]
    pub fn new(dir: &str, cfg: &DbsCfg) -> Hub {
        let files = Self::glob_files(dir);

        let db_sql_mapping = Self::build_schema_sql_files_mapping(files);

        let dbs = Self::build_dbs(cfg, db_sql_mapping);

        info!("Hub created");

        Hub { dbs }
    }

    fn glob_files(dir: &str) -> Vec<String> {
        let pattern = format!("{}/*.sql", dir);

        glob::glob(pattern.as_str())
            .unwrap()
            .map(|entry| entry.unwrap().to_str().unwrap().to_string())
            .collect::<Vec<_>>()
    }

    fn build_schema_sql_files_mapping(files: Vec<String>) -> HashMap<String, Vec<String>> {
        files
            .into_iter()
            .into_group_map_by(|f| f.split('-').nth(1).unwrap().to_string())
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    v.into_iter()
                        .sorted_by_key(|f| f.clone())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<HashMap<String, Vec<String>>>()
    }

    fn build_dbs(
        cfg: &DbsCfg,
        db_sql_mapping: HashMap<String, Vec<String>>,
    ) -> HashMap<String, DB> {
        db_sql_mapping
            .into_iter()
            .map(|(schema, sql_files)| {
                (
                    schema.clone(),
                    DB {
                        schema: schema.clone(),
                        secret: cfg
                            .dbs
                            .get(&schema)
                            .unwrap_or_else(|| panic!("not found schema {} in cfg", schema.clone()))
                            .secret
                            .clone(),
                        sql_files,
                    },
                )
            })
            .collect::<HashMap<String, DB>>()
    }
}

#[cfg(test)]
mod hub_tests {
    use super::*;

    #[test]
    fn glob_test_files_success() {
        let result = Hub::glob_files("./tests/sqlfiles");

        assert_eq!(result.len(), 4);
    }

    #[test]
    fn group_by_value_with_order() {
        let data = vec![
            "3-db1-test.sql".to_string(),
            "1-db1-test.sql".to_string(),
            "3-db2-test.sql".to_string(),
            "1-db2-test.sql".to_string(),
        ];
        let res = Hub::build_schema_sql_files_mapping(data);

        assert_eq!(
            res,
            HashMap::from([
                (
                    "db1".to_string(),
                    vec!["1-db1-test.sql".to_string(), "3-db1-test.sql".to_string()]
                ),
                (
                    "db2".to_string(),
                    vec!["1-db2-test.sql".to_string(), "3-db2-test.sql".to_string()]
                ),
            ])
        );
    }

    #[test]
    fn build_dbs_success() {
        use crate::cfg::DbCfg;
        let cfg = DbsCfg {
            dbs: HashMap::from([
                (
                    "db1".to_string(),
                    DbCfg {
                        schema: "db1".to_string(),
                        secret: "secret1".to_string(),
                    },
                ),
                (
                    "db2".to_string(),
                    DbCfg {
                        schema: "db2".to_string(),
                        secret: "secret2".to_string(),
                    },
                ),
                (
                    "db3".to_string(),
                    DbCfg {
                        schema: "db3".to_string(),
                        secret: "secret3".to_string(),
                    },
                ),
            ]),
        };
        let hub = Hub::new("./tests/sqlfiles", &cfg);
        assert_eq!(hub.dbs.len(), 3);
    }
}
