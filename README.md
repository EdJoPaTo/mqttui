# MQTT TUI

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

#### Arch Linux (AUR)

`paru -S mqttui-bin` or `yay -S mqttui-bin`

#### Other

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
MQTT TUI 0.14.0
EdJoPaTo <mqttui-rust@edjopato.de>
Subscribe to a MQTT Topic or publish something quickly from the terminal

USAGE:
    mqttui [OPTIONS] [TOPIC] [SUBCOMMAND]

ARGS:
    <TOPIC>
            Topic to watch

            [env: MQTTUI_TOPIC=]
            [default: #]

OPTIONS:
    -b, --broker <HOST>
            Host on which the MQTT Broker is running

            [env: MQTTUI_BROKER=]
            [default: localhost]

    -h, --help
            Print help information

    -p, --port <INT>
            Port on which the MQTT Broker is running

            [env: MQTTUI_PORT=]
            [default: 1883]

        --password <STRING>
            Password to access the mqtt broker. Passing the password via command line is
            insecure as the password can be read from the history!

            [env: MQTTUI_PASSWORD=]

    -u, --username <STRING>
            Username to access the mqtt broker. Anonymous access when not supplied.

            [env: MQTTUI_USERNAME=]

    -V, --version
            Print version information

SUBCOMMANDS:
    help
            Print this message or the help of the given subcommand(s)
    publish
            Publish a value quickly [aliases: p, pub]
```

```plaintext
mqttui-publish
Publish a value quickly

USAGE:
    mqttui publish [OPTIONS] <TOPIC> <PAYLOAD>

ARGS:
    <TOPIC>
            Topic to publish to

    <PAYLOAD>
            Payload to be published

OPTIONS:
    -b, --broker <HOST>
            Host on which the MQTT Broker is running

            [env: MQTTUI_BROKER=]
            [default: localhost]

    -h, --help
            Print help information

    -p, --port <INT>
            Port on which the MQTT Broker is running

            [env: MQTTUI_PORT=]
            [default: 1883]

        --password <STRING>
            Password to access the mqtt broker. Passing the password via command line is
            insecure as the password can be read from the history!

            [env: MQTTUI_PASSWORD=]

    -r, --retain
            Publish the MQTT message retained

            [env: MQTTUI_RETAIN=]

    -u, --username <STRING>
            Username to access the mqtt broker. Anonymous access when not supplied.

            [env: MQTTUI_USERNAME=]

    -v, --verbose
            Show full MQTT communication
```

Tip: Create an alias for the broker you are working on:
```bash
alias mqttui-home='mqttui --broker pi-home.local'

# Use the alias without having to specify the broker every time
mqttui-home "topic"
```

You can also create an environment variable for this:
```bash
export MQTTUI_BROKER=pi-home.local

# Use the command without specifying the broker every time
mqttui "topic
```

# Interesting Alternatives

- [thomasnordquist/MQTT-Explorer](https://github.com/thomasnordquist/MQTT-Explorer)
- [hivemq/mqtt-cli](https://github.com/hivemq/mqtt-cli)
