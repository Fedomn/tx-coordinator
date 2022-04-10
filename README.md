# tx-coordinator

Transaction Coordinator for SQL execution in different database

## Features

- guaranteed different database statements in a logic transaction

## Install

download binary from [release](https://github.com/Fedomn/tx-coordinator/releases)

## Usage

```shell
./txcoordinator --cfg ./cfg.toml --dir ./sqlfiles
```

### cfg.toml example

```yaml
[[databases]]
schema = "db1"
secret = "postgres://postgres:@127.0.0.1/db1"

[[databases]]
schema = "db2"
secret = "postgres://postgres:@127.0.0.1/db2"

[[databases]]
schema = "db3"
secret = "postgres://postgres:@127.0.0.1/db3"
```

### sqlfiles example

filename pattern: `{sql_index}-{database_name}-{table_name}.sql`

```sql
0-db1-t0.sql
1-db1-t1.sql
0-db2-t.sql
0-db3-t.sql
```

It groups sqlfiles using `database_name` and executes sql in ascending order of `sql_index`. In above example, the ececution order of db1 is *0-db1-t0.sql, 1-db1-t1.sql*

## Simulation operation

- `make db` to start integrated PostgreSQL
- `make simulate` to simulate real environment rollback operation