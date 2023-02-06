mod beam;
mod blaze;
use base64::{engine::general_purpose, Engine as _};
use serde_json::{from_slice, from_value};

use crate::errors::SpotError;

mod errors;

#[tokio::main]
async fn main() -> Result<(), SpotError> {

   beam::check_availability().await;
   blaze::check_availability().await;

   loop {
       match process_tasks().await {
           Ok(_) => (),
           Err(e) => {
               println!("Error while processing tasks: {}", e);
           }
       }
   }
}

async fn process_tasks() -> Result<(), SpotError> {
    println!("Start processing tasks...");

    let result = beam::retrieve_tasks().await;

    match result {
        Ok(tasks) => {
            for task in tasks {
                println!("Process task with ID: {}", task.id);

                let inquery = parse_inquery(task.to_owned());
                match inquery {
                    Ok(query) => {
                        let run_result = run_inquery(task.to_owned(), query).await;
                        match run_result {
                            Ok(run_result_result) => {
                                let answer_result =
                                    beam::answer_task(task, run_result_result).await;
                                match answer_result {
                                    Ok(()) => {
                                        return Ok(());
                                    }
                                    Err(e) => {
                                        return Err(e);
                                    }
                                }
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
        Err(e) => {
            return Err(e);
        }
    };

    Ok(())
}

fn parse_inquery(task: beam::BeamTask) -> Result<beam::Inquery, SpotError> {
    let decoded = general_purpose::STANDARD
        .decode(task.body)
        .map_err(|e| SpotError::DecodeError(e));
    match decoded {
        Ok(decoded_content) => {
            let inquery: beam::Inquery = from_slice(&decoded_content).unwrap();
            Ok(inquery)
        }
        Err(e) => {
            return Err(e);
        }
    }
}

async fn run_inquery(
    task: beam::BeamTask,
    inquery: beam::Inquery,
) -> Result<beam::BeamResult, SpotError> {
    if inquery.lang == "cql" {
        return run_cql_query(task, inquery).await;
    } else {
      return Ok(beam::BeamResult::perm_failed(
         beam::APP_ID.to_owned() + "." + beam::PROXY_ID,
         vec![task.from],
         task.id,
         format!("Can't run inqueries with language {}", inquery.lang),
         beam::APP_ID.to_owned(),
     ));
    }
}

async fn run_cql_query(
    task: beam::BeamTask,
    inquery: beam::Inquery,
) -> Result<beam::BeamResult, SpotError> {
    let cql_result = blaze::run_cql_query(inquery.lib, inquery.measure).await;
    match cql_result {
        Ok(text) => {
            let result = beam_result(task.to_owned(), text);
            match result {
                Ok(beam_result) => {
                    return Ok(beam_result);
                }
                Err(e) => {
                  return Ok(beam::BeamResult::perm_failed(
                     beam::APP_ID.to_owned() + "." + beam::PROXY_ID,
                     vec![task.to_owned().from],
                     task.to_owned().id,
                     e.to_string(),
                     beam::APP_ID.to_owned(),
                 ));
                }
            }
        }
        Err(e) => {
         return Ok(beam::BeamResult::perm_failed(
            beam::APP_ID.to_owned() + "." + beam::PROXY_ID,
            vec![task.to_owned().from],
            task.to_owned().id,
            e.to_string(),
            beam::APP_ID.to_owned(),
         ));
        }
    }
}

fn beam_result(
    task: beam::BeamTask,
    measure_report: String,
) -> Result<beam::BeamResult, SpotError> {
    let data = general_purpose::STANDARD.encode(measure_report.as_bytes());
    return Ok(beam::BeamResult::succeeded(
        beam::APP_ID.to_owned() + "." + beam::PROXY_ID,
        vec![task.from],
        task.id,
        data,
        beam::APP_ID.to_owned(),
    ));
}
