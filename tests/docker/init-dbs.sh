#!/bin/bash

set -eu

function create_user_and_database() {
	local database=$1
	echo "  Creating user and database '$database'"
	psql <<-EOSQL
	    CREATE USER $database;
	    CREATE DATABASE $database;
	    GRANT ALL PRIVILEGES ON DATABASE $database TO $database;
EOSQL
}

function init_schema_tables() {
	local database=$1
  echo "  Init schema and tables '$database'"
	psql --username "$database" $database <<-EOSQL
	    CREATE SCHEMA IF NOT EXISTS $database AUTHORIZATION $database;
      ALTER USER $database SET search_path to $database;
EOSQL
  psql --username "$database" $database <<-EOSQL
      CREATE TABLE $database.test_table (id INTEGER primary key);
EOSQL
}

echo "Multiple database creation requested: $POSTGRES_MULTIPLE_DATABASES_USERS"
for db in $(echo $POSTGRES_MULTIPLE_DATABASES_USERS | tr ',' ' '); do
  create_user_and_database $db
done
echo "Multiple databases created"

echo "  Init schema and tables requested: $POSTGRES_MULTIPLE_DATABASES_USERS"
for db in $(echo $POSTGRES_MULTIPLE_DATABASES_USERS | tr ',' ' '); do
  init_schema_tables $db
done
echo "Multiple databases initialized"
