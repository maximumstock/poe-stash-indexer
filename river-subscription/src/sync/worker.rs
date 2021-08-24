use std::{io::Write, sync::mpsc::Sender};

use crate::common::StashTabResponse;

use super::{state::SharedState, IndexerMessage};

pub(crate) fn start_worker(
    shared_state: SharedState,
    tx: Sender<IndexerMessage>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || loop {
        let mut lock = shared_state.lock().unwrap();

        if lock.should_stop {
            tx.send(IndexerMessage::Stop).unwrap();
            return;
        }

        if let Some(mut task) = lock.worker_queue.pop_front() {
            let start = std::time::Instant::now();
            let mut buffer = Vec::new();
            buffer.write_all(&task.fetch_partial).unwrap();
            task.reader.read_to_end(&mut buffer).unwrap();
            log::debug!("Took {}ms to read body", start.elapsed().as_millis());

            let start = std::time::Instant::now();
            let deserialized = serde_json::from_slice::<StashTabResponse>(&buffer)
                .expect("Deserialization of body failed");
            log::debug!("Took {}ms to deserialize body", start.elapsed().as_millis());

            let msg = IndexerMessage::Tick {
                payload: deserialized,
                change_id: task.change_id,
                created_at: std::time::SystemTime::now(),
            };

            tx.send(msg).expect("Sending IndexerMessage::Tick failed");
        } else {
            drop(lock);
            log::debug!("Worker is waiting due to no work");
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    })
}
