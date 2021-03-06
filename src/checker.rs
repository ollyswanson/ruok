//! # Checker
//!
//! Periodically make an HTTP GET request to each [`Service`]. If the service responds with 200 OK
//! then the service is considered to be up, otherwise it is considered to be down. If the
//! [`Status`] of the service has changed then a message is sent to [`NotifierHandle`] with the
//! name of the service and the new status.

use crate::notifier::{NotifierHandle, NotifierMsg};
use crate::service::Service;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};
use tokio::{sync::mpsc, task::JoinHandle};

// TODO: Find out whether we need a way to shut down the interval.
/// Starts a `tokio::time::interval` which periodically sends a message to `Checker` with the name
/// of the service that needs to be checked.
fn start_check_interval(name: &'static str, interval: u64, send: mpsc::Sender<CheckerMsg>) {
    use CheckerMsg::CheckService;

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(interval));

        loop {
            interval.tick().await;
            // Do nothing with a failed send for now
            let _ = send.send(CheckService(name)).await;
        }
    });
}

struct Checker {
    client: Client,
    /// Handle of the `Notifier` actor
    notifier: NotifierHandle,
    /// Receives the messages sent by the timer intervals
    receiver: mpsc::Receiver<CheckerMsg>,
    /// List of services to check.
    services: &'static HashMap<String, Service>,
    /// Tracks the [`Status`] of each [`Service`], if the status changes then a message will be
    /// sent to the `Notifer` actor.
    statuses: Arc<Mutex<HashMap<String, Status>>>,
}

enum CheckerMsg {
    /// Message sent by a spawned interval to [`Checker`] telling it to check the status of the
    /// specified service.
    CheckService(&'static str),
}

impl Checker {
    fn new(
        client: Client,
        notifier: NotifierHandle,
        receiver: mpsc::Receiver<CheckerMsg>,
        services: &'static HashMap<String, Service>,
    ) -> Self {
        let statuses: HashMap<_, _> = services
            .keys()
            .cloned()
            .map(|key| (key, Status::Up))
            .collect();

        Self {
            client,
            notifier,
            receiver,
            services,
            statuses: Arc::new(Mutex::new(statuses)),
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                CheckerMsg::CheckService(name) => {
                    self.check_service(name);
                }
            }
        }
    }

    fn check_service(&self, name: &'static str) {
        let services = self.services;
        let notifier = self.notifier.clone();
        let statuses = self.statuses.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            // Ok to unwrap, should never receive the name of a service that does not exist.
            let url = &services.get(name).unwrap().url;
            // for now we will produce a Status::Down for all other responses / errors.
            let status = match client.get(url.clone()).send().await {
                Ok(res) if res.status() == 200 => Status::Up,
                _ => Status::Down,
            };

            let notification: Option<NotifierMsg> = {
                let mut statuses = statuses.try_lock().expect("Failed to acquire Mutex lock");
                let current_status = statuses.get_mut(name).unwrap();

                if status != *current_status {
                    *current_status = status;
                    Some(NotifierMsg::Notify { name, status })
                } else {
                    None
                }
            };

            if let Some(notification) = notification {
                // Ignore the result from sending the notification
                let _ = notifier.sender.send(notification).await;
            }
        });
    }
}

// TODO: CheckerHandle differs in purpose from NotifierHandle despite sharing a similar name, have
// a think about a better way to describe it.
/// Contains a [`JoinHandle`] to be awaited when starting the main loop for the application.
pub struct CheckerHandle {
    pub join_handle: JoinHandle<()>,
}

impl CheckerHandle {
    /// Constructs the `Checker` and then starts the timer intervals that send messages to
    /// `Checker` prompting it to reach out to the specified services using the `client`.
    pub fn new(
        client: Client,
        notifier: NotifierHandle,
        services: &'static HashMap<String, Service>,
    ) -> Self {
        // TODO: Find a more suitable bound for the channel.
        let (tx, rx) = mpsc::channel(32);
        let intervals: Vec<_> = services
            .iter()
            .map(|(name, service)| (name, service.interval))
            .collect();

        // Start the checker before starting the intervals as each interval will send a message
        // immediately upon construction.
        let mut checker = Checker::new(client, notifier, rx, services);
        let handle = tokio::spawn(async move { checker.run().await });

        for (name, interval) in intervals {
            start_check_interval(name, interval, tx.clone());
        }

        Self {
            join_handle: handle,
        }
    }
}

/// Status of a service, if the endpoint returns and OK 200 then the `Status` is `Up`, otherwise it
/// is `Down`
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Status {
    Up,
    Down,
}
