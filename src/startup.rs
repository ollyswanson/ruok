use crate::checker::CheckerHandle;
use crate::config::Service;
use crate::notifications::{Notification, SlackNotification};
use crate::notifier::NotifierHandle;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use url::Url;

static SERVICES: OnceCell<HashMap<String, Service>> = OnceCell::new();
static NOTIFICATIONS: OnceCell<HashMap<String, Notification>> = OnceCell::new();

pub async fn startup() {
    let client = reqwest::Client::new();
    // arbitrary channel size based on Tokio tutorial.

    let mut services = HashMap::new();
    services.insert(
        "localhost".into(),
        Service {
            url: Url::parse("http://localhost:3000/health_check").unwrap(),
            interval: 2,
            notifications: vec!["test".into()],
        },
    );

    let mut notifications = HashMap::new();
    notifications.insert(
        "test".into(),
        Notification::Slack(SlackNotification {
            url: Url::parse("http://localhost:3000/slack").unwrap(),
        }),
    );

    SERVICES.set(services).unwrap();
    NOTIFICATIONS.set(notifications).unwrap();

    let notifier = NotifierHandle::new(
        client.clone(),
        SERVICES.get().unwrap(),
        NOTIFICATIONS.get().unwrap(),
    );
    let checker = CheckerHandle::new(client.clone(), notifier, SERVICES.get().unwrap());
    checker.join_handle.await.unwrap();
}
