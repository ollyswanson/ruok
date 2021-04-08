# Ruok

A very basic HTTP status checker implemented for fun with asynchronous Rust and
an attempt at the actor model.

## Configuration

Syntax / terminology inspired by `docker-compose`.

### Basic example

The services and notifications are defined in a configuration file such as
`config.yaml`.

```yaml
# Services to check
services:
  service_1:
    url: http://localhost:3000/health_check
    # Time interval in seconds (will improve granularity soon)
    interval: 5
    notfications: [notification_1]
  smooth:
    url: http://localhost:3000/annie
    interval: 258
    notifications: [notification_1]

# Where and how to send notifications
notifications:
  notification_1:
    type: slack
    url: http://foo.bar/baz
```

## Run

The application can be run with the command `ruok ./config.yaml`.

## Todo

- [ ] Basic authentication for the services.
- [ ] Better error handling.
- [ ] Add integration tests using something like `mockito`.
- [ ] Better documentation.
- [ ] Logging.
- [ ] More types of notifications.
