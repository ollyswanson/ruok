use crate::notifier::{NotifierHandle, NotifierMsg};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

// TODO: Find out whether we need a way to shut down the interval.
/// Starts a `tokio::time::interval` which periodically sends a message to `Checker` with the name
/// of the service that needs to be checked.
fn start_check_interval(name: String, interval: u64, send: mpsc::Sender<CheckerMsg>) {
    use CheckerMsg::CheckService;

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(interval));

        loop {
            interval.tick().await;
            // Do nothing with a failed send for now
            let _ = send.send(CheckService(name.clone())).await;
        }
    });
}

struct Checker {
    client: reqwest::Client,
    notifier: NotifierHandle,
    receiver: mpsc::Receiver<CheckerMsg>,
    services: Arc<HashMap<String, Service>>,
    statuses: Arc<Mutex<HashMap<String, Status>>>,
}

enum CheckerMsg {
    CheckService(String),
}

impl Checker {
    fn new(
        client: reqwest::Client,
        notifier: NotifierHandle,
        receiver: mpsc::Receiver<CheckerMsg>,
        services: HashMap<String, Service>,
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
            services: Arc::new(services),
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

    fn check_service(&self, name: String) {
        let services = self.services.clone();
        let notifier = self.notifier.clone();
        let statuses = self.statuses.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            // Ok to unwrap, should never receive the name of a service that does not exist.
            let url = &services.get(&name).unwrap().url;
            // for now we will produce a Status::Down for all other responses / errors.
            let status = match client.get(url).send().await {
                Ok(res) if res.status() == 200 => Status::Up,
                _ => Status::Down,
            };

            // The code here is a bit weird because of using a scope to make sure that the Mutex
            // guard is dropped.
            let notification: Option<NotifierMsg> = {
                let mut statuses = statuses.try_lock().expect("Failed to acquire Mutex lock");
                let current_status = statuses.get_mut(&name).unwrap();

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

// Don't strictly need a handle at the moment as we have nothing external that will be sending
// messages to `Checker`.
pub struct CheckerHandle {
    sender: mpsc::Sender<CheckerMsg>,
}

impl CheckerHandle {
    /// Constructs the `Checker` and then starts the timer intervals that send messages to
    /// `Checker` prompting it to reach out to the specified services using client.
    pub fn new(
        client: reqwest::Client,
        notifier: NotifierHandle,
        services: HashMap<String, Service>,
    ) -> Self {
        // Arbitrary size, need to do more investigation of what is appropriate.
        let (tx, rx) = mpsc::channel(32);
        let intervals: Vec<_> = services
            .iter()
            .map(|(name, service)| (name.clone(), service.interval))
            .collect();

        let mut checker = Checker::new(client, notifier, rx, services);
        tokio::spawn(async move { checker.run().await });

        for (name, interval) in intervals {
            start_check_interval(name, interval, tx.clone());
        }

        Self { sender: tx }
    }
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Status {
    Up,
    Down,
}

// Move somewhere else
pub struct Service {
    url: String,
    interval: u64,
}
