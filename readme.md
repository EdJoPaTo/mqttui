# MQTT TUI
![Rust](https://github.com/EdJoPaTo/mqttui/workflows/Rust/badge.svg)

> Subscribe to a MQTT Topic or publish something quickly from the terminal

![Screenshot](media/screenshot.jpg)

Taking a look into existing "lets just view MQTT right now" or "quickly publish something" projects they are always quite bulky and not that fast.

Currently I stick with [thomasnordquist/MQTT-Explorer](https://github.com/thomasnordquist/MQTT-Explorer) as it has a great overview of whats going on, a small topic based history and a sorted main view.
But having it running its eating up a lot of resources.

Quickly publish something from command line is also not that fun.
The feature rich cli alternative [hivemq/mqtt-cli](https://github.com/hivemq/mqtt-cli) takes some time to do its job and is not as easy to use as it has a lot of flags to specify.
Subscribing to many topics also isnt as useful to watch at as I hoped for.

Thats why I started building my own terminal based version to quickly watch or publish MQTT stuff.
It wont be as feature rich as something like the hivemq approach but it aims at being easy to use and fast.

## Install

### Prebuilt

Check the [Releases](https://github.com/EdJoPaTo/mqttui/releases).

### From Source

- Clone this repository
- `cargo install --path .`

## Usage

```sh
# Subscribe to everything (#)
mqttui

# Subscribe to topic
mqttui "topic"

# Subscribe to topic with a specific host (default is localhost)
mqttui -h "test.mosquitto.org" "hello/world"

# Publish to topic
mqttui publish "topic" "payload"

# Publish to topic with a specific host
mqttui publish -h "test.mosquitto.org" "topic" "payload"
```

```plaintext
MQTT TUI 0.5.0
EdJoPaTo <mqttui-rust@edjopato.de>
Subscribe to a MQTT Topic or publish something quickly from the terminal

USAGE:
    mqttui [FLAGS] [OPTIONS] [ARGS]

FLAGS:
        --help           Prints help information
    -i, --interactive    Start in interactive mode
    -V, --version        Prints version information
    -v, --verbose        Show full MQTT communication

OPTIONS:
    -h, --host <HOST>    Host on which the MQTT Broker is running [default: localhost]
    -p, --port <INT>     Port on which the MQTT Broker is running [default: 1883]

ARGS:
    <TOPIC>      Topic to watch or publish to [default: #]
    <PAYLOAD>    (optional) Payload to be published. If none is given it is instead subscribed to the topic.
```

Tip: Create an alias for the host you are working on:
```bash
alias mqttui-home='mqttui -h pi-home.local'

# Use the alias without having to specify the host every time
mqttui-home "topic"
```

# Interesting Alternatives

- [thomasnordquist/MQTT-Explorer](https://github.com/thomasnordquist/MQTT-Explorer)
- [hivemq/mqtt-cli](https://github.com/hivemq/mqtt-cli)
