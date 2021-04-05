mod slack;
pub use slack::SlackNotification;

pub enum Notification {
    Slack(slack::SlackNotification),
}
