use crate::notifications::Notification;
use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

// TODO: Find a better home for me.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Service {
    pub url: Url,
    pub interval: u64,
    pub notifications: Vec<String>,
}

#[derive(Deserialize)]
struct UncheckedConfig {
    pub services: HashMap<String, Service>,
    pub notifications: HashMap<String, Notification>,
}

pub struct Config {
    pub services: &'static HashMap<String, Service>,
    pub notifications: &'static HashMap<String, Notification>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_deserialize_unchecked() {
        let yaml = r#"
            services:
                s1:
                    url: "http://localhost:3000/health_check"
                    interval: 2
                    notifications: [n1, n2]
                s2:
                    url: "http://localhost:3001/health_check"
                    interval: 3
                    notifications: [n1, n2]

            notifications:
                n1:
                    type: slack
                    url: "http://localhost:3000/slack"
                n2:
                    type: slack
                    url: "http://localhost:3001/slack"
            "#;

        let config: UncheckedConfig = serde_yaml::from_str(yaml).unwrap();
        let s1 = Service {
            url: Url::parse("http://localhost:3000/health_check").unwrap(),
            interval: 2,
            notifications: vec!["n1".into(), "n2".into()],
        };

        let s2 = Service {
            url: Url::parse("http://localhost:3001/health_check").unwrap(),
            interval: 3,
            notifications: vec!["n1".into(), "n2".into()],
        };

        assert_eq!(&s1, config.services.get("s1").unwrap());
        assert_eq!(&s2, config.services.get("s2").unwrap());
    }
}
