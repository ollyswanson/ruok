mod slack;
pub use slack::SlackNotification;

#[derive(Clone, Debug)]
pub enum Notification {
    Slack(slack::SlackNotification),
}

impl Notification {
    pub fn notify(&self) {
        println!("notification");
    }
}
