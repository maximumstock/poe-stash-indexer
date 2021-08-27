use std::{
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
};

use super::fetcher::{FetchTask, FetcherMessage};

pub(crate) enum SchedulerMessage {
    Task(FetchTask),
    Stop,
}

pub(crate) fn start_scheduler(
    scheduler_rx: Receiver<SchedulerMessage>,
    fetcher_tx: Sender<FetcherMessage>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        while let Ok(msg) = scheduler_rx.recv() {
            match msg {
                SchedulerMessage::Stop => break,
                SchedulerMessage::Task(task) => {
                    fetcher_tx
                        .send(FetcherMessage::Task(task))
                        .expect("scheduler: Failed to send FetcherMessage");
                }
            }
        }

        log::debug!("Shut down scheduler");
    })
}
