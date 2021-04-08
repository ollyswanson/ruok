use crate::checker::CheckerHandle;
use crate::config::Config;
use crate::notifications::Notification;
use crate::notifier::NotifierHandle;
use crate::service::Service;
use once_cell::sync::OnceCell;
use reqwest::Client;
use std::collections::HashMap;

static SERVICES: OnceCell<HashMap<String, Service>> = OnceCell::new();
static NOTIFICATIONS: OnceCell<HashMap<String, Notification>> = OnceCell::new();

/// Make [`Config`] static and start main checker loop.
pub async fn startup(client: Client, config: Config) {
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
