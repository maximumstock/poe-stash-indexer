use std::{collections::VecDeque, sync::Arc, time::Duration};

use bytes::BytesMut;
use futures::channel::mpsc::Sender;
use futures::Future;
use futures::{channel::mpsc::Receiver, lock::Mutex};

use crate::common::{poe_ninja_client::PoeNinjaClient, ChangeId, StashTabResponse};

#[derive(Default)]
pub struct Indexer {
    pub(crate) is_stopping: bool,
    pub(crate) jobs: Vec<Box<dyn Future<Output = u32>>>,
}

pub type IndexerResult = Receiver<IndexerMessage>;

#[cfg(feature = "async")]
impl Indexer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stop(&mut self) {
        self.is_stopping = true;
        log::info!("Stopping indexer");
    }

    pub fn is_stopping(&self) -> bool {
        self.is_stopping
    }

    /// Start the indexer with a given change_id
    pub async fn start_with_id(&mut self, change_id: ChangeId) -> IndexerResult {
        log::info!("Resuming at change id: {}", change_id);
        self.start(change_id).await
    }

    /// Start the indexer with the latest change_id from poe.ninja
    pub async fn start_with_latest(&mut self) -> IndexerResult {
        let latest_change_id = PoeNinjaClient::fetch_latest_change_id_async()
            .await
            .expect("Fetching lastest change_id from poe.ninja failed");
        log::info!("Fetched latest change id: {}", latest_change_id);
        self.start(latest_change_id).await
    }

    async fn start(&mut self, change_id: ChangeId) -> IndexerResult {
        let jobs = Arc::new(Mutex::new(VecDeque::from(vec![change_id])));
        let (tx, rx) = futures::channel::mpsc::channel(10);

        schedule_job(jobs, tx);
        rx
    }
}

fn schedule_job(jobs: Arc<Mutex<VecDeque<ChangeId>>>, tx: Sender<IndexerMessage>) {
    tokio::spawn(async move { process(jobs, tx).await });
}

async fn process(
    jobs: Arc<Mutex<VecDeque<ChangeId>>>,
    mut tx: Sender<IndexerMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(change_id) = jobs.lock().await.pop_back() {
        let jobs = jobs.clone();
        let url = format!(
            "http://www.pathofexile.com/api/public-stash-tabs?id={}",
            &change_id
        );

        // todo: static client somewhere
        let client = reqwest::ClientBuilder::new().build()?;
        let mut response = client.get(url).send().await?;

        let mut bytes = BytesMut::new();
        while let Some(chunk) = response.chunk().await? {
            bytes.extend_from_slice(&chunk);

            if bytes.len() > 120 {
                let next_change_id = parse_next_change_id(&bytes).await.unwrap();
                jobs.lock().await.push_back(next_change_id);
                schedule_job(jobs.clone(), tx.clone());
            }
        }

        let response = StashTabResponse {
            next_change_id: "der".into(),
            stashes: vec![],
        };
        tx.try_send(IndexerMessage::Tick {
            response,
            change_id,
            created_at: std::time::SystemTime::now(),
        })
        .expect("Sending IndexerMessage failed");
    }

    Ok(())
}

async fn parse_next_change_id(bytes: &BytesMut) -> Result<ChangeId, Box<dyn std::error::Error>> {
    Err("derp".into())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexerMessage {
    Tick {
        response: StashTabResponse,
        change_id: ChangeId,
        created_at: std::time::SystemTime,
    },
    RateLimited(Duration),
    Stop,
}
