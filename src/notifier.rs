//! # Notifier
//!
//! Receives a message to send a notification and then sends the appropriate notification.
use crate::checker::Status;
use crate::notifications::Notification;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct NotifierHandle {
    pub sender: mpsc::Sender<NotifierMsg>,
}

pub enum NotifierMsg {
    Notify { name: String, status: Status },
}
