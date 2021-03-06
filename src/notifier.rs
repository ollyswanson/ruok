//! # Notifier
//!
//! Responsible for receiving messages from `Checker` and sending the corresponding
//! [`Notification`]
use crate::checker::Status;
use crate::notifications::Notification;
use crate::service::Service;
use reqwest::Client;
use std::collections::HashMap;
use tokio::sync::mpsc;

struct Notifier {
    client: Client,
    receiver: mpsc::Receiver<NotifierMsg>,
    services: &'static HashMap<String, Service>,
    notifications: &'static HashMap<String, Notification>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NotifierMsg {
    Notify { name: &'static str, status: Status },
}

impl Notifier {
    fn new(
        client: Client,
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
        client: Client,
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
