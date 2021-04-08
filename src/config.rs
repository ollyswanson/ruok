use crate::notifications::Notification;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::BufRead;
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
    pub services: HashMap<String, Service>,
    pub notifications: HashMap<String, Notification>,
}

impl Config {
    // TODO: Replace the boxed error with something more helpful
    pub fn new<R: BufRead>(reader: R) -> Result<Self, Box<dyn std::error::Error>> {
        let unchecked = serde_yaml::from_reader(reader)?;

        Self::parse(unchecked)
    }

    fn parse(unchecked: UncheckedConfig) -> Result<Self, Box<dyn std::error::Error>> {
        for (name, service) in unchecked.services.iter() {
            if service
                .notifications
                .iter()
                .any(|notification| unchecked.notifications.get(notification).is_none())
            {
                return Err(format!("Service: {} has undefined notification", name))?;
            }
        }

        Ok(Self {
            services: unchecked.services,
            notifications: unchecked.notifications,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

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

        let cursor = Cursor::new(yaml.as_bytes());
        let config = Config::new(cursor).unwrap();

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

    #[test]
    fn should_be_err_if_undefined_notfication() {
        let yaml = r#"
            services:
                s1:
                    url: "http://localhost:3000/health_check"
                    interval: 2
                    notifications: [n1, n2]

            notifications:
                n1:
                    type: slack
                    url: "http://localhost:3000/slack"
            "#;
        let cursor = Cursor::new(yaml.as_bytes());

        // TODO: update the test to check the type of error once the boxed error has been replaced.
        assert!(Config::new(cursor).is_err());
    }
}
