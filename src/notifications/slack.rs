use crate::checker::Status;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SlackNotification {
    // FIX: Remove pub once we've created a deserialize for Self
    pub url: String,
}

impl SlackNotification {
    pub async fn notify(&self, client: reqwest::Client, name: String, status: Status) {
        let payload = self.build_payload(name, status);

        // TODO: Do something with the Result
        let _ = client.post(&self.url).json(&payload).send().await;
    }

    // TODO: Make the notification much nicer.
    fn build_payload(&self, name: String, status: Status) -> HashMap<String, String> {
        let mut payload = HashMap::new();

        let text = match status {
            Status::Up => format!("{} is up :thumbsup:", &name),
            Status::Down => format!("{} is down :thumbsdown:", &name),
        };

        payload.insert("text".into(), text);

        payload
    }
}
