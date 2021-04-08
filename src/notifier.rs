//! # Notifier
//!
//! Receives a message to send a notification and then sends the appropriate notification.
use crate::checker::Status;
use crate::config::Service;
use crate::notifications::Notification;
use std::collections::HashMap;
use tokio::sync::mpsc;

struct Notifier {
    client: reqwest::Client,
    receiver: mpsc::Receiver<NotifierMsg>,
    services: &'static HashMap<String, Service>,
    notifications: &'static HashMap<String, Notification>,
}

pub enum NotifierMsg {
    Notify { name: &'static str, status: Status },
}

impl Notifier {
    fn new(
        client: reqwest::Client,
        receiver: mpsc::Receiver<NotifierMsg>,
        services: &'static HashMap<String, Service>,
        notifications: &'static HashMap<String, Notification>,
    ) -> Self {
        Self {
            client,
            receiver,
            services,
            notifications,
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                NotifierMsg::Notify { name, status } => {
                    self.handle_notification(name, status);
                }
            }
        }
    }

    fn handle_notification(&self, name: &'static str, status: Status) {
        for notif_name in &self.services.get(name).unwrap().notifications {
            let notification = self.notifications.get(notif_name).unwrap();

            // Delegating the spawning of threads to the notifications
            notification.send(self.client.clone(), name, status);
        }
    }
}

#[derive(Clone)]
pub struct NotifierHandle {
    pub sender: mpsc::Sender<NotifierMsg>,
}

impl NotifierHandle {
    pub fn new(
        client: reqwest::Client,
        services: &'static HashMap<String, Service>,
        notifications: &'static HashMap<String, Notification>,
    ) -> Self {
        // TODO: Find a suitable bound for the channel.
        let (tx, rx) = mpsc::channel(32);
        let mut notifier = Notifier::new(client, rx, services, notifications);

        tokio::spawn(async move {
            notifier.run().await;
        });

        Self { sender: tx }
    }
}
