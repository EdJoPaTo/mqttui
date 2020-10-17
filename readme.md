# MQTT CLI
![Rust](https://github.com/EdJoPaTo/mqtt-cli/workflows/Rust/badge.svg)

> Subscribe to a MQTT Topic or publish something quickly from the terminal

## Install

- Clone this repository
- `cargo install --path .`

## Usage

```sh
# Subscribe to everything (#)
mqtt

# Subscribe to topic
mqtt "topic"

# Publish to topic
mqtt "topic" "payload"

# Subscribe to topic with a specific host (default is localhost)
mqtt -h "test.mosquitto.org" "hello/world"
```

```plaintext
Quick MQTT CLI 0.1.0
EdJoPaTo <mqtt-cli-rust@edjopato.de>
Small Command Line Utility to quickly publish or subscribe something to a given mqtt topic

USAGE:
    mqtt [FLAGS] [OPTIONS] <TOPIC> [PAYLOAD]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Show full MQTT communication

OPTIONS:
    -h, --host <HOST>    Host on which the MQTT Broker is running [default: localhost]
    -p, --port <INT>     Port on which the MQTT Broker is running [default: 1883]

ARGS:
    <TOPIC>      Topic to watch or publish to
    <PAYLOAD>    (optional) Payload to be published. If none is given it is instead subscribed to the topic.
```
