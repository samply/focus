use std::{rc::Rc, time::Duration};

use futures_util::{future::LocalBoxFuture, FutureExt, Stream, StreamExt};
use tracing::{error, warn, debug, Instrument, info_span};

use crate::{errors::FocusError, beam, BeamTask, BeamResult};

const NUM_WORKERS: usize = 3;

pub async fn process_tasks<F>(task_handler: F)
where
    F: Fn(&BeamTask) -> LocalBoxFuture<'_, Result<BeamResult, FocusError>> + Clone + 'static,
{
    stream_task_results(task_handler)
        .buffer_unordered(NUM_WORKERS)
        .for_each_concurrent(None, |(task, task_result)| answer_task_result(task, task_result))
        .await
}

fn stream_task_results<F>(on_task: F) -> impl Stream<Item = LocalBoxFuture<'static, (BeamTask, Result<BeamResult, FocusError>)>>
where
    F: Fn(&BeamTask) -> LocalBoxFuture<'_, Result<BeamResult, FocusError>> + Clone + 'static,
{
    let on_task_claimed = |res| if let Err(e) = res {
        warn!("Failed to claim task: {e}");
    } else {
        debug!("Successfully claimed task");
    };
    futures_util::stream::repeat_with(beam::retrieve_tasks)
        .filter_map(|v| async {
            match v.await {
                Ok(mut ts) => ts.pop(),
                Err(e) => {
                    warn!("Failed to get tasks from beam: {e}");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    None
                },
            }
        })
        .then(move |t| {
            let id = t.id;
            let span = info_span!("task", %id);
            let span_for_handler = span.clone();
            let task = Rc::new(t);
            let on_task = on_task.clone();
            async move {
                let task1 = Rc::clone(&task);
                let task2 = Rc::clone(&task);
                let mut task_claiming = std::pin::pin!(beam::claim_task(&task2));
                let mut task_processing = async move {
                    on_task(&task1).instrument(span_for_handler).await
                }.boxed_local();
                tokio::select! {
                    task_processed = &mut task_processing => {
                        tracing::debug!("Proccessed task before claimed");
                        on_task_claimed(task_claiming.as_mut().await);
                        futures_util::future::ready((Rc::try_unwrap(task).unwrap(), task_processed)).boxed_local()
                    },
                    task_claimed = &mut task_claiming => {
                        on_task_claimed(task_claimed);
                        task_processing.map(|v| (Rc::try_unwrap(task).unwrap(), v)).boxed_local()
                    }
                }
            }.instrument(span)
        })
}

async fn answer_task_result(task: BeamTask, task_result: Result<BeamResult, FocusError>) {
    let result = match task_result {
        Ok(res) => res,
        Err(e) => {
            warn!("Failed to execute query: {e}");
            if let Err(e) = beam::fail_task(&task, e.user_facing_error()).await {
                warn!("Failed to report failure to beam: {e}");
            }
            return;
        }
    };

    const MAX_TRIES: u32 = 150;
    for attempt in 0..MAX_TRIES {
        match beam::answer_task(&result).await {
            Ok(_) => break,
            Err(FocusError::ConfigurationError(s)) => {
                error!(
                    "FATAL: Unable to report back to Beam due to a configuration issue: {s}"
                );
            }
            Err(FocusError::UnableToAnswerTask(e)) => {
                warn!("Unable to report task result to Beam: {e}. Retrying (attempt {attempt}/{MAX_TRIES}).");
            }
            Err(e) => {
                warn!("Unknown error reporting task result back to Beam: {e}. Retrying (attempt {attempt}/{MAX_TRIES}).");
            }
        };
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
