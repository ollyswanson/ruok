use crate::checker::{CheckerHandle, Service};
use crate::notifier::NotifierHandle;
use std::collections::HashMap;
use tokio::sync::mpsc;

pub async fn startup() {
    // TODO
    // Parse Config
    // Create Notifiers
    // Create Checkers, pass Notifier handle to Checkers
    // Create Intervals, pass Checker handles to Intervals
    // await intervals
    let client = reqwest::Client::new();
    // arbitrary channel size based on Tokio tutorial.
    let (tx, rx) = mpsc::channel(32);

    let mut services = HashMap::new();
    services.insert(
        "localhost".into(),
        Service {
            url: "http://localhost:3000/health_check".into(),
            interval: 2,
        },
    );
    let notifier = NotifierHandle { sender: tx };
    let checker = CheckerHandle::new(client, notifier, services);
    checker.join_handle.await.unwrap();
}
