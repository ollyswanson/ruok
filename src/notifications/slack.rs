use crate::checker::Status;
use reqwest::Url;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct SlackNotification {
    pub url: Url,
}

impl SlackNotification {
    pub async fn notify(&self, client: reqwest::Client, name: &'static str, status: Status) {
        let payload = self.build_payload(name, status);

        // TODO: Do something with the Result
        let _ = client.post(self.url.clone()).json(&payload).send().await;
    }

    // TODO: Make the notification much nicer.
    fn build_payload(&self, name: &'static str, status: Status) -> HashMap<String, String> {
        let mut payload = HashMap::new();

        let text = match status {
            Status::Up => format!("{} is up :thumbsup:", name),
            Status::Down => format!("{} is down :thumbsdown:", name),
        };

        payload.insert("text".into(), text);

        payload
    }
}
