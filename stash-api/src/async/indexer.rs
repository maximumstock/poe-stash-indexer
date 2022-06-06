use std::time::Duration;

use crate::common::{poe_ninja_client::PoeNinjaClient, ChangeId, StashTabResponse};

#[derive(Default)]
pub struct Indexer {
    // pub(crate) scheduler_tx: Option<Sender<SchedulerMessage>>,
    pub(crate) is_stopping: bool,
}

#[cfg(feature = "async")]
impl Indexer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stop(&mut self) {
        self.is_stopping = true;
        log::info!("Stopping indexer");
        // self.scheduler_tx
        //     .as_ref()
        //     .expect("indexer: Missing ref to scheduler_rx")
        //     .send(SchedulerMessage::Stop)
        //     .expect("indexer: Failed to send SchedulerMessage::Stop");
    }

    pub fn is_stopping(&self) -> bool {
        self.is_stopping
    }

    /// Start the indexer with a given change_id
    pub fn start_with_id(&mut self, change_id: ChangeId) -> () {
        log::info!("Resuming at change id: {}", change_id);
        self.start(change_id)
    }

    /// Start the indexer with the latest change_id from poe.ninja
    pub async fn start_with_latest(&mut self) -> () {
        let latest_change_id = PoeNinjaClient::fetch_latest_change_id_async()
            .await
            .expect("Fetching lastest change_id from poe.ninja failed");
        log::info!("Fetched latest change id: {}", latest_change_id);
        self.start(latest_change_id)
    }

    fn start(&mut self, change_id: ChangeId) -> () {
        // let (indexer_rx, scheduler_tx) = start_scheduler();

        // scheduler_tx
        //     .send(SchedulerMessage::Fetch(FetchTask::new(change_id)))
        //     .expect("indexer: Failed to schedule initial FetchTask");

        // self.scheduler_tx = Some(scheduler_tx);

        // indexer_rx
        ()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexerMessage {
    Tick {
        payload: StashTabResponse,
        change_id: ChangeId,
        created_at: std::time::SystemTime,
    },
    RateLimited(Duration),
    Stop,
}

#[cfg(test)]
mod test {
    use std::{sync::mpsc::RecvTimeoutError, time::Duration};

    use super::{Indexer, IndexerMessage};

    #[test]
    fn test_indexer() {}
}
