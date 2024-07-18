use crate::errors::FocusError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, postgres::PgRow, PgPool};
use sqlx_pgrow_serde::SerMapPgRow;
use std::collections::HashMap;
use tracing::{error, info, debug};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SqlQuery {
    pub payload: String,
}

include!(concat!(env!("OUT_DIR"), "/sql_replace_map.rs"));

pub async fn get_pg_connection_pool(pg_url: &str, num_attempts: u32) -> Result<PgPool, FocusError> {
    info!("Trying to establish a PostgreSQL connection pool");

    let mut attempts = 0;
    let mut err: Option<FocusError> = None;

    while attempts < num_attempts {
        info!(
            "Attempt to connect to PostgreSQL {} of {}",
            attempts + 1,
            num_attempts
        );
        match PgPoolOptions::new()
            .max_connections(10)
            .connect(pg_url)
            .await
        {
            Ok(pg_con_pool) => {
                info!("PostgreSQL connection successfull");
                return Ok(pg_con_pool);
            }
            Err(e) => {
                error!(
                    "Failed to connect to PostgreSQL. Attempt {} of {}: {}",
                    attempts + 1,
                    num_attempts,
                    e
                );
                err = Some(FocusError::CannotConnectToDatabase(e.to_string()));
            }
        }
        attempts += 1;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    Err(err.unwrap_or_else(|| {
        FocusError::CannotConnectToDatabase("Failed to connect to PostgreSQL".into())
    }))
}

pub async fn healthcheck(pool: &PgPool) -> bool {
    let res = run_query(pool, SQL_REPLACE_MAP.get("SELECT_TABLES").unwrap()).await; //this file exists, safe to unwrap
    res.is_ok()
}

pub async fn run_query(pool: &PgPool, query: &str) -> Result<Vec<PgRow>, FocusError> {
    sqlx::query(query)
        .fetch_all(pool)
        .await
        .map_err(FocusError::ErrorExecutingQuery)
}

pub async fn process_sql_task(pool: &PgPool, key: &str) -> Result<Vec<PgRow>, FocusError> {
    debug!("Executing query with key = {}", &key);
    let sql_query = SQL_REPLACE_MAP.get(&key);
    let Some(query) = sql_query else {
        return Err(FocusError::QueryNotAllowed(key.into()));
    };
    debug!("Executing query {}", &query);

    run_query(pool, query).await
}

pub fn serialize_rows(rows: Vec<PgRow>) -> Result<Value, FocusError> {
    let mut rows_json: Vec<Value> = vec![];

    for row in rows {
        let row = SerMapPgRow::from(row);
        let row_json = serde_json::to_value(&row)?;
        rows_json.push(row_json);
    }

    Ok(json!(rows_json))
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    #[ignore] //TODO mock DB
    async fn connect_healthcheck() {
        let pool =
            get_pg_connection_pool("postgresql://postgres:secret@localhost:5432/postgres", 1)
                .await
                .unwrap();

        assert!(healthcheck(&pool).await);
    }

    #[tokio::test]
    #[ignore] //TODO mock DB
    async fn serialize() {
        let pool =
            get_pg_connection_pool("postgresql://postgres:secret@localhost:5432/postgres", 1)
                .await
                .unwrap();

        let rows = run_query(&pool, SQL_REPLACE_MAP.get("SELECT_TABLES").unwrap())
            .await
            .unwrap();

        let rows_json = serialize_rows(rows).unwrap();

        assert!(rows_json.is_array());

        assert_ne!(rows_json[0]["hasindexes"], Value::Null);
    }
}
