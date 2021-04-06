mod slack;
pub use slack::SlackNotification;

pub enum Notification {
    Slack(slack::SlackNotification),
}

impl Notification {
    pub fn notify(&self) {
        println!("notification");
    }
}
