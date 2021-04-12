use httpmock::MockServer;
use reqwest::Client;
use ruok::config::Config;
use ruok::startup;
use std::io::Cursor;
use tokio::time::{self, Duration};

macro_rules! advance_time {
    ($a: expr) => {{
        tokio::time::advance(tokio::time::Duration::from_millis($a - 10u64)).await;
        tokio::time::resume();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        tokio::time::pause();
    }};
}

#[tokio::test]
async fn integration() {
    let server = MockServer::start();
    let raw_config = create_config(&server.base_url());
    let config = Config::new(Cursor::new(raw_config)).unwrap();
    let client = Client::new();

    // set up mocks
    let mut s1 = server.mock(|when, then| {
        when.path("/s1");
        then.status(200);
    });
    let mut s2 = server.mock(|when, then| {
        when.path("/s2");
        then.status(404);
    });
    let n1 = server.mock(|when, then| {
        when.path("/n1");
        then.status(200);
    });
    let n2 = server.mock(|when, then| {
        when.path("/n2");
        then.status(200);
    });
    let n3 = server.mock(|when, then| {
        when.path("/n3");
        then.status(200);
    });

    tokio::spawn(async move {
        startup::startup(client, config).await;
    });

    // Allow some time to pass so that requests etc are sent and received by mock server.
    time::sleep(Duration::from_millis(10)).await;
    // From here we will pause time and advance the clock with the tools tokio provides, unfreezing
    // and sleeping for a small amount of time to allow interaction with the mock server.
    time::pause();

    // services are both checked as soon as the intervals start.
    s1.assert_hits(1);
    s2.assert_hits(1);
    // s1 is ok so no notifications sent to n1.
    n1.assert_hits(0);
    // s2 was down so notifications sent to n2 and n3.
    n2.assert_hits(1);
    n3.assert_hits(1);

    // s2 comes back online but the count will have reset.
    s2.delete();
    s2 = server.mock(|when, then| {
        when.path("/s2");
        then.status(200);
    });

    // 1 second has passed.
    advance_time!(1000);
    // 1 second has passed so s1 should have been checked a second time.
    s1.assert_hits(2);
    // s2 has a 2 second interval so it won't have been checked again yet.
    s2.assert_hits(0);

    // 2 seconds have passed.
    advance_time!(1000);
    s1.assert_hits(3);
    // 2 seconds have passed so s2 should be being checked for a second time.
    s2.assert_hits(1);
    // s1 has been up the entire time so n1 should have no notifications.
    n1.assert_hits(0);
    // s2 is back up so n2 and n3 should have new notifications to let them know.
    n2.assert_hits(2);
    n3.assert_hits(2);

    // s1 has gone down.
    s1.delete();
    s1 = server.mock(|when, then| {
        when.path("/s1");
        then.status(404);
    });

    // 3 seconds have passed.
    advance_time!(1000);
    // count has reset.
    s1.assert_hits(1);
    s2.assert_hits(1);
    // n1 and n2 have been notified that s1 is down.
    n1.assert_hits(1);
    n2.assert_hits(3);
    n3.assert_hits(2);

    // s1 has come back up.
    s1.delete();
    s1 = server.mock(|when, then| {
        when.path("/s1");
        then.status(200);
    });

    // 4 seconds have passed.
    advance_time!(1000);
    s1.assert_hits(1);
    s2.assert_hits(2);
    n1.assert_hits(2);
    n2.assert_hits(4);
    n3.assert_hits(2);
}

// TODO: Is there a nicer way to do this?
fn create_config(base_url: &str) -> String {
    let mut config = String::new();
    config.push_str(&format!(
        r#"
        services:
            s1:
                url: {}/s1 
                interval: 1
                notifications: [n1, n2]
        "#,
        base_url
    ));
    config.push_str(&format!(
        r#"
            s2:
                url: {}/s2
                interval: 2
                notifications: [n2, n3]
        "#,
        base_url
    ));
    config.push_str(&format!(
        r#"
        notifications:
            n1:
                type: slack
                url: {}/n1 
        "#,
        base_url
    ));
    config.push_str(&format!(
        r#"
            n2:
                type: slack
                url: {}/n2 
        "#,
        base_url
    ));
    config.push_str(&format!(
        r#"
            n3:
                type: slack
                url: {}/n3 
        "#,
        base_url
    ));

    config
}
