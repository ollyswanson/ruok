use crate::checker::Status;
use reqwest::Client;
use serde::Deserialize;

mod slack;
pub use slack::SlackNotification;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Notification {
    #[serde(rename = "slack")]
    Slack(slack::SlackNotification),
}

impl Notification {
    pub fn send(&'static self, client: Client, name: &'static str, status: Status) {
        match self {
            Self::Slack(notification) => {
                tokio::spawn(async move {
                    notification.send(client, name, status).await;
                });
            }
        }
    }
}
