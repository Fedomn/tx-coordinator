use std::collections::HashMap;

use anyhow::Result;
use itertools::Itertools;

/// Hub organizes database schema and sql files.
pub struct Hub {
    dbs: HashMap<String, Vec<String>>,
}

impl Hub {
    pub fn new(dir: &str) -> Result<Hub> {
        let pattern = format!("{}/*.sql", dir);

        let files = glob::glob(pattern.as_str())?
            .map(|entry| entry.unwrap().to_str().unwrap().to_string())
            .collect::<Vec<_>>();

        let dbs = Hub::build_schema_sql_files_mapping(files);

        Ok(Hub { dbs })
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
}

#[cfg(test)]
mod hub_tests {
    use super::*;

    #[test]
    fn glob_test_files_success() {
        let result = Hub::new("./tests/sqlfiles").unwrap();
        assert_eq!(
            result.dbs,
            HashMap::from([
                (
                    "db1".to_string(),
                    vec![
                        "tests/sqlfiles/0-db1-test.sql".to_string(),
                        "tests/sqlfiles/1-db1-test.sql".to_string(),
                    ]
                ),
                (
                    "db2".to_string(),
                    vec!["tests/sqlfiles/0-db2-test.sql".to_string()]
                ),
                (
                    "db3".to_string(),
                    vec!["tests/sqlfiles/0-db3-test.sql".to_string()]
                ),
            ])
        );
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
}
