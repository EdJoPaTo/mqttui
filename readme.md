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
MQTT CLI 0.2.0
EdJoPaTo <mqtt-cli-rust@edjopato.de>
Subscribe to a MQTT Topic or publish something quickly from the terminal

USAGE:
    mqtt [FLAGS] [OPTIONS] [ARGS]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Show full MQTT communication

OPTIONS:
    -h, --host <HOST>    Host on which the MQTT Broker is running [default: localhost]
    -p, --port <INT>     Port on which the MQTT Broker is running [default: 1883]

ARGS:
    <TOPIC>      Topic to watch or publish to [default: #]
    <PAYLOAD>    (optional) Payload to be published. If none is given it is instead subscribed to the topic.
```

Tip: Create an alias for the host you are working on:
```bash
alias mqtt-home='mqtt -h pi-home.local'

# Use the alias without having to specify the host every time
mqtt-home "topic" "payload"
```
