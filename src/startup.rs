use crate::checker::{CheckerHandle, Service};
use crate::notifier::NotifierHandle;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use tokio::sync::mpsc;

static SERVICES: OnceCell<HashMap<String, Service>> = OnceCell::new();

pub async fn startup() {
    let client = reqwest::Client::new();
    // arbitrary channel size based on Tokio tutorial.
    let (tx, rx) = mpsc::channel(32);

    let mut services = HashMap::new();
    services.insert(
        "localhost".into(),
        Service {
            url: "http://localhost:3000/health_check".into(),
            interval: 2,
            notifications: vec!["test".into()],
        },
    );

    SERVICES.set(services).unwrap();

    let notifier = NotifierHandle { sender: tx };
    let checker = CheckerHandle::new(client, notifier, SERVICES.get().unwrap());
    checker.join_handle.await.unwrap();
}
