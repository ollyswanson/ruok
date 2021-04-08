use serde::Deserialize;
use url::Url;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Service {
    /// Url of health_check endpoint for the service.
    pub url: Url,
    // TODO: Review whether to update unit to milliseconds and allow specification of unit by
    // allowing a unit suffix in the config file.
    /// Interval in seconds for checking the endpoint.
    pub interval: u64,
    /// List of names of the notifications to send for the given service.
    pub notifications: Vec<String>,
}
