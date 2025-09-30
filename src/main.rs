mod ast;
mod banner;
mod beam;
mod blaze;
mod config;
mod cql;
mod errors;
mod graceful_shutdown;
mod logger;

mod eucaim_api;
mod exporter;
mod intermediate_rep;
mod mr;
mod projects;
mod task_processing;
mod transformed;
mod util;

#[cfg(feature = "query-sql")]
mod db;

#[cfg(feature = "query-sql")]
mod eucaim_sql;

#[cfg(feature = "query-sql")]
use sqlx::Row;

use base64::engine::general_purpose;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use beam_lib::{TaskRequest, TaskResult};
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use laplace_rs::ObfCache;
use tokio::sync::Mutex;

use crate::blaze::{parse_blaze_query_payload_ast, AstQuery};
use crate::config::EndpointType;
use crate::util::{base64_decode, is_cql_tampered_with, obfuscate_counts_mr};
use crate::{config::CONFIG, errors::FocusError};
use blaze::CqlQuery;

use std::collections::{HashMap, HashSet};
use std::ops::DerefMut;
use std::process::ExitCode;
use std::str;
use std::sync::Arc;
use std::time::Instant;
use std::{process::exit, time::Duration};

use serde::{Deserialize, Serialize};
use tracing::{debug, error, trace, warn};

// result cache
type SearchQuery = String;
type Obfuscated = bool;
type QueryResult = String;
type BeamTask = TaskRequest<String>;
type BeamResult = TaskResult<beam_lib::RawString>;

#[derive(Deserialize, Debug)]
#[serde(tag = "lang", rename_all = "lowercase")]
enum Language {
    #[cfg(not(feature = "bbmri"))]
    Cql(CqlQuery),
    Ast(AstQuery),
}

#[derive(Clone, PartialEq, Debug, Copy, Serialize, Deserialize, Eq, Hash, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum Transform {
    Lens,
    #[default]
    None,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Metadata {
    project: String,
    task_type: Option<exporter::TaskType>,
    #[serde(default)]
    transform: Transform,
}

#[derive(Debug, Clone, Default)]
struct QueryResultCache {
    queries_to_cache: HashSet<String>,
    cache: HashMap<(SearchQuery, Obfuscated, Transform), (QueryResult, Instant)>,
}

impl QueryResultCache {
    const TTL: Duration = Duration::from_secs(24 * 60 * 60); //24h

    pub fn new() -> Self {
        if let Some(filename) = &CONFIG.queries_to_cache {
            match std::fs::read_to_string(filename) {
                Ok(content) => {
                    let queries_to_cache = content
                        .lines()
                        .map(ToOwned::to_owned)
                        .collect::<HashSet<String>>();
                    return Self {
                        cache: Default::default(),
                        queries_to_cache,
                    };
                }
                Err(e) => {
                    warn!(
                        "Cannot read queries to cache from file {}: {e}",
                        filename.display()
                    );
                }
            };
        }

        Self {
            cache: Default::default(),
            queries_to_cache: Default::default(),
        }
    }

    pub fn insert(&mut self, key: (SearchQuery, Obfuscated, Transform), value: QueryResult) {
        let created = Instant::now();
        self.cache.insert(key, (value, created));
    }

    pub fn get(&self, key: &(SearchQuery, Obfuscated, Transform)) -> QueryResultCacheOutcome {
        if !self.queries_to_cache.contains(&key.0) {
            return QueryResultCacheOutcome::DontCache;
        }
        if let Some((result, created)) = self.cache.get(key) {
            if Instant::now().duration_since(*created) < Self::TTL {
                return QueryResultCacheOutcome::Cached(result);
            }
        }
        QueryResultCacheOutcome::ShouldCache
    }
}

#[must_use]
pub enum QueryResultCacheOutcome<'a> {
    Cached(&'a QueryResult),
    ShouldCache,
    DontCache,
}

#[derive(Serialize)]
struct EucaimResponse {
    collections: Vec<Collection>,
    total: TotalCount,
    provider: String,
    provider_icon: String,
}

#[derive(Serialize)]
struct Collection {
    age_range: AgeRange,
    body_parts: Vec<String>,
    description: String,
    gender: Vec<String>,
    id: String,
    modalities: Vec<String>,
    name: String,
    studies_count: i32,
    subjects_count: i32,
}

#[derive(Serialize)]
struct AgeRange {
    min: u8,
    max: u8,
}

#[derive(Serialize)]
struct TotalCount {
    studies_count: i32,
    subjects_count: i32,
}

#[tokio::main]
pub async fn main() -> ExitCode {
    if let Err(e) = logger::init_logger() {
        error!("Cannot initalize logger: {}", e);
        exit(1);
    };
    banner::print_banner();

    trace!("WARNING: You are running Focus in trace logging. This log level outputs unobfuscated result counts and is only intended for debugging the obfuscation. To avoid privacy risks, please check if that log level is appropriate. Consider using \"info\" or \"warn\".");

    let _ = CONFIG.api_key; // Initialize config

    tokio::select! {
        _ = graceful_shutdown::wait_for_signal() => {
            ExitCode::SUCCESS
        },
        code = main_loop() => {
            code
        }
    }
}

#[cfg(not(feature = "query-sql"))]
type DbPool = ();

#[cfg(feature = "query-sql")]
type DbPool = sqlx::PgPool;

#[cfg(not(feature = "query-sql"))]
async fn get_db_pool() -> Result<Option<DbPool>, ExitCode> {
    Ok(None)
}

#[cfg(feature = "query-sql")]
async fn get_db_pool() -> Result<Option<DbPool>, ExitCode> {
    use tracing::info;

    if let Some(connection_string) = CONFIG.postgres_connection_string.clone() {
        match db::get_pg_connection_pool(&connection_string, CONFIG.max_db_attempts).await {
            Err(e) => {
                error!("Error connecting to database: {}", e);
                Err(ExitCode::from(8))
            }

            Ok(pool) => {
                info!("Postgresql connection established");
                Ok(Some(pool))
            }
        }
    } else {
        Ok(None)
    }
}

async fn main_loop() -> ExitCode {
    let db_pool = match get_db_pool().await {
        Ok(pool) => pool,
        Err(code) => {
            return code;
        }
    };
    let endpoint_service_available: fn() -> BoxFuture<'static, bool> = match CONFIG.endpoint_type {
        EndpointType::Blaze => || blaze::check_availability().boxed(),
        EndpointType::Omop | EndpointType::EucaimApi => || async { true }.boxed(), // TODO health check
        #[cfg(feature = "query-sql")]
        EndpointType::EucaimSql => || async { true }.boxed(),
        #[cfg(feature = "query-sql")]
        EndpointType::BlazeAndSql => || blaze::check_availability().boxed(),
        #[cfg(feature = "query-sql")]
        EndpointType::Sql => || async { true }.boxed(),
    };
    let mut failures = 0;
    while !(beam::check_availability().await && endpoint_service_available().await) {
        failures += 1;
        if failures >= CONFIG.retry_count {
            error!(
                "Encountered too many errors -- exiting after {} attempts.",
                CONFIG.retry_count
            );
            return ExitCode::from(22);
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
        warn!(
            "Retrying connection (attempt {}/{})",
            failures, CONFIG.retry_count
        );
    }
    let query_result_cache = Arc::new(Mutex::new(QueryResultCache::new()));
    let obf_cache = Arc::new(Mutex::new(ObfCache {
        cache: Default::default(),
    }));
    task_processing::process_tasks(move |task| {
        let obf_cache = obf_cache.clone();
        let query_result_cache = query_result_cache.clone();
        process_task(task, obf_cache, query_result_cache, db_pool.clone()).boxed_local()
    })
    .await;
    ExitCode::FAILURE
}

async fn process_task(
    task: &BeamTask,
    obf_cache: Arc<Mutex<ObfCache>>,
    query_result_cache: Arc<Mutex<QueryResultCache>>,
    db_pool: Option<DbPool>,
) -> Result<BeamResult, FocusError> {
    debug!("Processing task {}", task.id);

    trace!("{}", &task.body);

    let metadata: Metadata = serde_json::from_value(task.metadata.clone()).unwrap_or(Metadata {
        project: "default_obfuscation".to_string(),
        task_type: None,
        transform: Transform::None,
    });

    debug!("{:?}", &metadata);

    if metadata.project == "focus-healthcheck" {
        return Ok(beam::beam_result::succeeded(
            CONFIG.beam_app_id_long.clone(),
            vec![task.from.clone()],
            task.id,
            "healthy".into(),
        ));
    }
    if metadata.project == "exporter" {
        let Some(task_type) = metadata.task_type else {
            return Err(FocusError::MissingExporterTaskType);
        };
        let body = &task.body;
        return run_exporter_query(task, body, task_type).await;
    }

    match CONFIG.endpoint_type {
        EndpointType::Blaze => {
            let mut generated_from_ast: bool = false;
            let data = base64_decode(&task.body)?;
            let query: CqlQuery = match serde_json::from_slice::<Language>(&data)? {
                #[cfg(not(feature = "bbmri"))]
                Language::Cql(cql_query) => {
                    if !CONFIG.enable_cql_lang {
                        return Err(FocusError::CqlLangNotEnabled);
                    }
                    cql_query
                }
                Language::Ast(ast_query) => {
                    generated_from_ast = true;
                    serde_json::from_str(&cql::generate_body(parse_blaze_query_payload_ast(
                        &ast_query.payload,
                    )?)?)?
                }
            };
            run_cql_query(
                task,
                &query,
                obf_cache,
                query_result_cache,
                metadata.project,
                metadata.transform,
                generated_from_ast,
            )
            .await
        }
        #[cfg(feature = "query-sql")]
        EndpointType::BlazeAndSql => {
            let mut generated_from_ast: bool = false;
            let data = base64_decode(&task.body)?;
            let query_maybe: Result<Language, serde_json::Error> = serde_json::from_slice(&data);
            if let Ok(cql_query) = query_maybe {
                let query = match cql_query {
                    #[cfg(not(feature = "bbmri"))]
                    Language::Cql(cql_query) => {
                        if !CONFIG.enable_cql_lang {
                            return Err(FocusError::CqlLangNotEnabled);
                        }
                        cql_query
                    }
                    Language::Ast(ast_query) => {
                        generated_from_ast = true;
                        serde_json::from_str(&cql::generate_body(parse_blaze_query_payload_ast(
                            &ast_query.payload,
                        )?)?)?
                    }
                };
                run_cql_query(
                    task,
                    &query,
                    obf_cache,
                    query_result_cache,
                    metadata.project,
                    metadata.transform,
                    generated_from_ast,
                )
                .await
            } else {
                let sql_query: db::SqlQuery = serde_json::from_slice(&data)?;
                if let Some(pool) = db_pool {
                    run_sql_key_query(task, pool, sql_query, query_result_cache).await
                } else {
                    Err(FocusError::CannotConnectToDatabase(
                        "SQL task but no connection String in config".into(),
                    ))
                }
            }
        }
        #[cfg(feature = "query-sql")]
        EndpointType::Sql => {
            let data = base64_decode(&task.body)?;
            let query_maybe: Result<db::SqlQuery, serde_json::Error> =
                serde_json::from_slice(&(data));
            if let Ok(sql_query) = query_maybe {
                if let Some(pool) = db_pool {
                    run_sql_key_query(task, pool, sql_query, query_result_cache).await
                } else {
                    Err(FocusError::CannotConnectToDatabase(
                        "SQL task but no connection String in config".into(),
                    ))
                }
            } else {
                warn!(
                    "Wrong type of query for an SQL only store: {}, {:?}",
                    CONFIG.endpoint_type, data
                );
                Ok(beam::beam_result::perm_failed(
                    CONFIG.beam_app_id_long.clone(),
                    vec![task.from.clone()],
                    task.id,
                    format!(
                        "Wrong type of query for an SQL only store: {}, {:?}",
                        CONFIG.endpoint_type, data
                    ),
                ))
            }
        }
        EndpointType::Omop => {
            let decoded = util::base64_decode(&task.body)?;
            let intermediate_rep_query: intermediate_rep::IntermediateRepQuery =
                serde_json::from_slice(&decoded)?;
            //TODO check that the language is ast
            let query_decoded = general_purpose::STANDARD
                .decode(intermediate_rep_query.query)
                .map_err(FocusError::DecodeError)?;
            let ast: ast::Ast = serde_json::from_slice(&query_decoded)?;

            Ok(run_intermediate_rep_query(task, ast).await?)
        }
        EndpointType::EucaimApi => {
            let decoded = util::base64_decode(&task.body)?;
            let intermediate_rep_query: intermediate_rep::IntermediateRepQuery =
                serde_json::from_slice(&decoded)?;
            //TODO check that the language is ast
            let query_decoded = general_purpose::STANDARD
                .decode(intermediate_rep_query.query)
                .map_err(FocusError::DecodeError)?;
            let ast: ast::Ast = serde_json::from_slice(&query_decoded)?;

            Ok(run_eucaim_api_query(task, ast).await?)
        }
        #[cfg(feature = "query-sql")]
        EndpointType::EucaimSql => {
            let decoded = util::base64_decode(&task.body)?;
            let intermediate_rep_query: intermediate_rep::IntermediateRepQuery =
                serde_json::from_slice(&decoded)?;
            //TODO check that the language is ast
            let query_decoded = general_purpose::STANDARD
                .decode(intermediate_rep_query.query)
                .map_err(FocusError::DecodeError)?;
            let ast: ast::Ast = serde_json::from_slice(&query_decoded)?;

            let sql_query_maybe = eucaim_sql::build_eucaim_sql_query(ast);
            if let Ok(sql_query) = sql_query_maybe {
                if let Some(pool) = db_pool {
                    run_eucaim_sql_query(task, pool, sql_query, query_result_cache).await
                } else {
                    Err(FocusError::CannotConnectToDatabase(
                        "SQL task but no connection String in config".into(),
                    ))
                }
            } else {
                warn!(
                    "Wrong type of query for an SQL only store: {}, {:?}",
                    CONFIG.endpoint_type, decoded
                );
                Ok(beam::beam_result::perm_failed(
                    CONFIG.beam_app_id_long.clone(),
                    vec![task.from.clone()],
                    task.id,
                    format!(
                        "Wrong type of query for an SQL only store: {}, {:?}",
                        CONFIG.endpoint_type, decoded
                    ),
                ))
            }
        }
    }
}

#[cfg(feature = "query-sql")]
async fn run_eucaim_sql_query(
    task: &TaskRequest<String>,
    pool: sqlx::Pool<sqlx::Postgres>,
    sql_query: String,
    query_result_cache: Arc<Mutex<QueryResultCache>>,
) -> Result<TaskResult<beam_lib::RawString>, FocusError> {
    let should_cache =
        match query_result_cache
            .lock()
            .await
            .get(&(sql_query.clone(), false, Transform::None))
        {
            QueryResultCacheOutcome::Cached(result) => {
                return Ok(beam::beam_result::succeeded(
                    CONFIG.beam_app_id_long.clone(),
                    vec![task.from.clone()],
                    task.id,
                    BASE64.encode(result),
                ));
            }
            QueryResultCacheOutcome::ShouldCache => true,
            QueryResultCacheOutcome::DontCache => false,
        };
    let result = db::process_sql_task(&pool, &(sql_query)).await;
    let provider_icon = CONFIG
        .provider_icon
        .as_deref()
        .unwrap_or(include_str!("../resources/default_provider_icon"));

    let mut response: EucaimResponse = EucaimResponse {
        collections: Vec::new(),
        total: TotalCount {
            studies_count: 0,
            subjects_count: 0,
        },
        provider: CONFIG.provider.clone().unwrap_or_default(),
        provider_icon: provider_icon.to_owned(),
    };
    let mut studies_count: i32 = 0;
    let mut subjects_count: i32 = 0;
    if let Ok(rows) = result {
        for row in rows {
            let collection: Collection = Collection {
                age_range: AgeRange { min: 0, max: 0 },
                body_parts: Vec::new(),
                description: row.get("description"),
                gender: Vec::new(),
                id: row.get("id"),
                modalities: Vec::new(),
                name: row.get("name"),
                studies_count: row.get("studies_count"),
                subjects_count: row.get("subjects_count"),
            };
            studies_count += collection.studies_count;
            subjects_count += collection.subjects_count;
            response.collections.push(collection);
        }
        response.total.studies_count = studies_count;
        response.total.subjects_count = subjects_count;

        let response_json: String = serde_json::to_string(&response)
            .map_err(|e| FocusError::SerializationError(e.to_string()))?;

        dbg!(&response_json);

        if should_cache {
            query_result_cache
                .lock()
                .await
                .insert((sql_query, false, Transform::None), response_json.clone());
        }

        Ok(beam::beam_result::succeeded(
            CONFIG.beam_app_id_long.clone(),
            vec![task.clone().from],
            task.id,
            BASE64.encode(serde_json::to_string(&response_json)?),
        ))
    } else {
        Err(FocusError::QueryResultBad(
            "Query executed but result not readable".into(),
        ))
    }
}

#[cfg(feature = "query-sql")]
async fn run_sql_key_query(
    task: &TaskRequest<String>,
    pool: sqlx::Pool<sqlx::Postgres>,
    sql_query: db::SqlQuery,
    query_result_cache: Arc<Mutex<QueryResultCache>>,
) -> Result<TaskResult<beam_lib::RawString>, FocusError> {
    let should_cache = match query_result_cache.lock().await.get(&(
        sql_query.payload.clone(),
        false,
        Transform::None,
    )) {
        QueryResultCacheOutcome::Cached(result) => {
            return Ok(beam::beam_result::succeeded(
                CONFIG.beam_app_id_long.clone(),
                vec![task.from.clone()],
                task.id,
                BASE64.encode(result),
            ));
        }
        QueryResultCacheOutcome::ShouldCache => true,
        QueryResultCacheOutcome::DontCache => false,
    };
    let result = db::process_sql_key_task(&pool, &(sql_query.payload)).await;
    if let Ok(rows) = result {
        let rows_json = db::serialize_rows(rows)?;

        if should_cache {
            query_result_cache.lock().await.insert(
                (sql_query.payload, false, Transform::None),
                rows_json.to_string(),
            );
        }

        Ok(beam::beam_result::succeeded(
            CONFIG.beam_app_id_long.clone(),
            vec![task.clone().from],
            task.id,
            BASE64.encode(serde_json::to_string(&rows_json)?),
        ))
    } else {
        Err(FocusError::QueryResultBad(
            "Query executed but result not readable".into(),
        ))
    }
}

async fn run_cql_query(
    task: &BeamTask,
    query: &CqlQuery,
    obf_cache: Arc<Mutex<ObfCache>>,
    query_result_cache: Arc<Mutex<QueryResultCache>>,
    project: String,
    transform: Transform,
    generated_from_ast: bool,
) -> Result<BeamResult, FocusError> {
    let encoded_query =
        query.lib["content"][0]["data"]
            .as_str()
            .ok_or(FocusError::ParsingError(format!(
                "Not a valid library: Field .content[0].data not found. Library: {}",
                query.lib
            )))?;

    let obfuscate =
        CONFIG.obfuscate == config::Obfuscate::Yes && !CONFIG.unobfuscated.contains(&project);

    let should_cache = match query_result_cache.lock().await.get(&(
        encoded_query.to_string(),
        obfuscate,
        transform,
    )) {
        QueryResultCacheOutcome::Cached(result) => {
            return Ok(beam::beam_result::succeeded(
                CONFIG.beam_app_id_long.clone(),
                vec![task.from.clone()],
                task.id,
                BASE64.encode(result),
            ));
        }
        QueryResultCacheOutcome::ShouldCache => true,
        QueryResultCacheOutcome::DontCache => false,
    };

    let query = if generated_from_ast {
        query.clone()
    } else {
        replace_cql_library(query.clone())?
    };

    trace!("Library: {}", &query.lib);
    trace!("Measure: {}", &query.measure);

    let cql_result = blaze::run_cql_query(&query.lib, &query.measure).await?;

    trace!("MeasureReport with unobfuscated values: {}", &cql_result);

    let cql_result_new: String = match obfuscate {
        true => obfuscate_counts_mr(
            &cql_result,
            obf_cache.lock().await.deref_mut(),
            CONFIG.obfuscate_zero,
            CONFIG.obfuscate_below_10_mode,
            CONFIG.delta_patient,
            CONFIG.delta_specimen,
            CONFIG.delta_diagnosis,
            CONFIG.delta_procedures,
            CONFIG.delta_medication_statements,
            CONFIG.delta_histo,
            CONFIG.epsilon,
            CONFIG.rounding_step,
        )?,
        false => cql_result,
    };

    let result_string = match transform {
        Transform::Lens => {
            let result_mr: mr::MeasureReport = serde_json::from_str(&cql_result_new)?;
            let result_json = mr::transform_lens(result_mr)?;
            serde_json::to_string(&result_json)
                .map_err(|e| FocusError::SerializationError(e.to_string()))?
        }
        Transform::None => cql_result_new,
    };

    if should_cache {
        query_result_cache.lock().await.insert(
            (encoded_query.to_string(), obfuscate, transform),
            result_string.clone(),
        );
    }

    let result = beam_result(task.to_owned(), result_string).unwrap_or_else(|e| {
        beam::beam_result::perm_failed(
            CONFIG.beam_app_id_long.clone(),
            vec![task.to_owned().from],
            task.to_owned().id,
            e.to_string(),
        )
    });

    Ok(result)
}

async fn run_intermediate_rep_query(
    task: &BeamTask,
    ast: ast::Ast,
) -> Result<BeamResult, FocusError> {
    let mut err = beam::beam_result::perm_failed(
        CONFIG.beam_app_id_long.clone(),
        vec![task.to_owned().from],
        task.to_owned().id,
        String::new(),
    );

    let mut intermediate_rep_result = intermediate_rep::post_ast(ast).await?;

    let provider_icon = CONFIG
        .provider_icon
        .clone()
        .unwrap_or(include_str!("../resources/default_provider_icon").to_string());

    intermediate_rep_result = intermediate_rep_result.replacen(
        '{',
        format!(r#"{{"provider_icon":"{}","#, provider_icon).as_str(),
        1,
    );

    let provider = CONFIG.provider.clone().unwrap_or_default();

    intermediate_rep_result = intermediate_rep_result.replacen(
        '{',
        format!(r#"{{"provider":"{}","#, provider).as_str(),
        1,
    );

    let result = beam_result(task.to_owned(), intermediate_rep_result).unwrap_or_else(|e| {
        err.body = beam_lib::RawString(e.to_string());
        err
    });

    Ok(result)
}

async fn run_eucaim_api_query(task: &BeamTask, ast: ast::Ast) -> Result<BeamResult, FocusError> {
    let mut err = beam::beam_result::perm_failed(
        CONFIG.beam_app_id_long.clone(),
        vec![task.to_owned().from],
        task.to_owned().id,
        String::new(),
    );

    let mut eucaim_api_query_result = eucaim_api::send_eucaim_api_query(ast).await?;

    let provider_icon = CONFIG
        .provider_icon
        .as_deref()
        .unwrap_or(include_str!("../resources/default_provider_icon"));

    eucaim_api_query_result = eucaim_api_query_result.replacen(
        '{',
        format!(r#"{{"provider_icon":"{}","#, provider_icon).as_str(),
        1,
    );

    let provider = CONFIG.provider.clone().unwrap_or_default();

    eucaim_api_query_result = eucaim_api_query_result.replacen(
        '{',
        format!(r#"{{"provider":"{}","#, provider).as_str(),
        1,
    );

    let result = beam_result(task.to_owned(), eucaim_api_query_result).unwrap_or_else(|e| {
        err.body = beam_lib::RawString(e.to_string());
        err
    });

    Ok(result)
}

async fn run_exporter_query(
    task: &BeamTask,
    body: &String,
    task_type: exporter::TaskType,
) -> Result<BeamResult, FocusError> {
    let mut err = beam::beam_result::perm_failed(
        CONFIG.beam_app_id_long.clone(),
        vec![task.to_owned().from],
        task.to_owned().id,
        String::new(),
    );

    let exporter_result = exporter::post_exporter_query(body, task_type).await?;

    let result = beam_result(task.to_owned(), exporter_result).unwrap_or_else(|e| {
        err.body = beam_lib::RawString(e.to_string());
        err
    });

    Ok(result)
}

fn replace_cql_library(mut query: CqlQuery) -> Result<CqlQuery, FocusError> {
    let old_data_value = &query.lib["content"][0]["data"];

    let old_data_string = old_data_value
        .as_str()
        .ok_or(FocusError::ParsingError(format!(
            "{} is not a valid library: Field .content[0].data not found.",
            query.lib
        )))?;

    let decoded_cql = util::base64_decode(old_data_string)?;

    let decoded_string = str::from_utf8(&decoded_cql)
        .map_err(|_| FocusError::ParsingError("CQL query was invalid".into()))?;

    match is_cql_tampered_with(decoded_string) {
        false => debug!("CQL not tampered with"),
        true => {
            debug!("CQL tampered with");
            return Err(FocusError::CQLTemperedWithError(
                "'define' keyword found in CQL".to_string(),
            ));
        }
    };

    let replaced_cql_str = util::replace_cql(decoded_string);
    let replaced_cql_str_base64 = BASE64.encode(replaced_cql_str);
    let new_data_value = serde_json::to_value(replaced_cql_str_base64)
        .expect("unable to turn base64 string into json value - this should not happen");

    let a = &mut query.lib["content"][0]["data"];
    *a = new_data_value;

    Ok(query)
}

fn beam_result(task: BeamTask, query_result: String) -> Result<BeamResult, FocusError> {
    let data = BASE64.encode(query_result.as_bytes());
    Ok(beam::beam_result::succeeded(
        CONFIG.beam_app_id_long.clone(),
        vec![task.from],
        task.id,
        data,
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    const METADATA_STRING: &str = r#"{"project": "exliquid"}"#;
    const METADATA_STRING_EXPORTER: &str = r#"{"project": "exporter", "task_type": "EXECUTE"}"#;

    #[test]
    fn test_metadata_deserialization_default() {
        let metadata: Metadata = serde_json::from_str(METADATA_STRING).unwrap_or(Metadata {
            project: "default_obfuscation".to_string(),
            task_type: None,
            transform: Transform::None,
        });

        assert_eq!(metadata.task_type, None);
    }

    #[test]
    fn test_metadata_deserialization_exporter() {
        let metadata: Metadata = serde_json::from_str(METADATA_STRING_EXPORTER).unwrap();

        assert_eq!(metadata.task_type, Some(exporter::TaskType::Execute));
    }
}
