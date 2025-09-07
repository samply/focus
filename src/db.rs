use crate::errors::FocusError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, postgres::PgRow, PgPool};
use sqlx_serde::SerMapPgRow;
use std::{collections::HashMap, time::Duration};
use tracing::{debug, info, warn};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SqlQuery {
    pub payload: String,
}

include!(concat!(env!("OUT_DIR"), "/sql_replace_map.rs"));

pub async fn get_pg_connection_pool(
    pg_url: &str,
    max_db_attempts: u32,
) -> Result<PgPool, FocusError> {
    info!("Trying to establish a PostgreSQL connection pool");

    tryhard::retry_fn(|| async {
        info!("Attempting to connect to PostgreSQL");
        PgPoolOptions::new()
            .max_connections(10)
            .connect(pg_url)
            .await
            .map_err(|e| {
                warn!("Failed to connect to PostgreSQL: {}", e);
                FocusError::CannotConnectToDatabase(e.to_string())
            })
    })
    .retries(max_db_attempts)
    .exponential_backoff(Duration::from_secs(2))
    .await
}

pub async fn run_query(pool: &PgPool, query: &str) -> Result<Vec<PgRow>, FocusError> {
    sqlx::query(query)
        .fetch_all(pool)
        .await
        .map_err(FocusError::ErrorExecutingSqlQuery)
}

pub async fn process_sql_key_task(pool: &PgPool, key: &str) -> Result<Vec<PgRow>, FocusError> {
    debug!("Executing query with key = {}", &key);
    let sql_query = SQL_REPLACE_MAP.get(&key);
    let Some(query) = sql_query else {
        return Err(FocusError::QueryNotAllowed(key.into()));
    };
    debug!("Executing query {}", &query);

    run_query(pool, query).await
}

pub async fn process_sql_task(pool: &PgPool, query: &str) -> Result<Vec<PgRow>, FocusError> {
    
    debug!("Executing query {}", &query);
    run_query(pool, query).await
}

pub fn serialize_rows(rows: Vec<PgRow>) -> Result<Value, FocusError> {
    let mut rows_json: Vec<Value> = Vec::with_capacity(rows.len());

    for row in rows {
        let row = SerMapPgRow::from(row);
        let row_json = serde_json::to_value(&row)?;
        rows_json.push(row_json);
    }

    Ok(Value::Array(rows_json))
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    #[ignore] //TODO mock DB
    async fn serialize() {
        let pool =
            get_pg_connection_pool("postgresql://postgres:secret@localhost:5432/postgres", 1)
                .await
                .unwrap();

        let rows = run_query(&pool, SQL_REPLACE_MAP.get("SELECT_TEST").unwrap())
            .await
            .unwrap();

        let rows_json = serialize_rows(rows).unwrap();

        assert!(rows_json.is_array());

        assert_ne!(rows_json[0]["floaty"], Value::Null);
    }
}
