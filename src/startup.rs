use crate::checker::CheckerHandle;
use crate::config::Config;
use crate::config::Service;
use crate::notifications::Notification;
use crate::notifier::NotifierHandle;
use once_cell::sync::OnceCell;
use std::collections::HashMap;

static SERVICES: OnceCell<HashMap<String, Service>> = OnceCell::new();
static NOTIFICATIONS: OnceCell<HashMap<String, Notification>> = OnceCell::new();

pub async fn startup(config: Config) {
    let client = reqwest::Client::new();
    // arbitrary channel size based on Tokio tutorial.

    SERVICES.set(config.services).unwrap();
    NOTIFICATIONS.set(config.notifications).unwrap();

    let notifier = NotifierHandle::new(
        client.clone(),
        SERVICES.get().unwrap(),
        NOTIFICATIONS.get().unwrap(),
    );

    let checker = CheckerHandle::new(client.clone(), notifier, SERVICES.get().unwrap());
    checker.join_handle.await.unwrap();
}
