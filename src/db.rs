use sqlx::{postgres::PgPoolOptions, PgPool, postgres::PgRow};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{error, info};
use crate::errors::FocusError;
use crate::util;

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
        info!("Attempt to connect to PostgreSQL {} of {}", attempts + 1, num_attempts);
        match PgPoolOptions::new()
            .max_connections(10)
            .connect(&pg_url)
            .await
        {
            Ok(pg_con_pool) => {
                info!("PostgreSQL connection successfull");
                return Ok(pg_con_pool)
            },
            Err(e) => {
                error!("Failed to connect to PostgreSQL. Attempt {} of {}: {}", attempts + 1, num_attempts, e);
                err = Some(FocusError::CannotConnectToDatabase(e.to_string()));
            }
        }
        attempts += 1;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    Err(err.unwrap_or_else(|| FocusError::CannotConnectToDatabase("Failed to connect to PostgreSQL".into())))
}

pub async fn healthcheck(pool: &PgPool) -> bool {

        let res = sqlx::query(include_str!("../resources/sql/SELECT_TABLES"))
            .fetch_all(pool)
            .await;
        if let Ok(_) = res {true} else {false}
}

pub async fn run_query(pool: &PgPool, query: &str) -> Result<Vec<PgRow>, FocusError> {

    sqlx::query(query)
        .fetch_all(pool)
        .await.map_err( FocusError::ErrorExecutingQuery)
}

pub async fn process_sql_task(pool: &PgPool, encoded: &str) -> Result<Vec<PgRow>, FocusError>{
    let decoded = util::base64_decode(encoded)?;
    let key = String::from_utf8(decoded).map_err(FocusError::ErrorConvertingToString)?;
    let key = key.as_str();
    let sql_query = SQL_REPLACE_MAP.get(&(key.clone()));
    if sql_query.is_none(){
        return Err(FocusError::QueryNotAllowed(key.into()));
    }
    let query = sql_query.unwrap(); 

    run_query(pool, query).await

}


#[cfg(test)] 
mod test {
    use super::*;

    #[tokio::test]
    #[ignore] //TODO mock DB
    async fn connect() {
        let pool = get_pg_connection_pool("postgresql://postgres:secret@localhost:5432/postgres", 1).await.unwrap();
    
        assert!(healthcheck(&pool).await);
    }
}

