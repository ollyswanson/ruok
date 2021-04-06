use crate::checker::Status;

mod slack;
pub use slack::SlackNotification;

#[derive(Clone, Debug)]
pub enum Notification {
    Slack(slack::SlackNotification),
}

impl Notification {
    pub fn notify(&'static self, client: reqwest::Client, name: &'static str, status: Status) {
        // REMOVE
        println!("notification");

        match self {
            Self::Slack(notification) => {
                tokio::spawn(async move {
                    notification.notify(client, name, status).await;
                });
            }
        }
    }
}
