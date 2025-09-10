use anyhow::Context;
use clap::{Args, Parser, Subcommand, ValueHint};
use url::Url;

#[allow(clippy::doc_markdown)]
#[derive(Debug, Subcommand)]
pub enum Subcommands {
    /// Clean retained messages from the broker.
    ///
    /// This works by subscribing to the topic and waiting for messages with the retained flag.
    /// Then a message with an empty payload is published retained which clears the topic on the broker.
    /// Ends on the first non retained message or when the timeout is reached.
    #[command(visible_alias = "c", visible_alias = "clean")]
    CleanRetained {
        /// Topic which gets cleaned.
        ///
        /// Supports filters like 'foo/bar/#'.
        #[arg(value_hint = ValueHint::Other)]
        topic: String,

        /// When there is no message received for the given time the operation is considered done
        #[arg(
            long,
            value_hint = ValueHint::Other,
            value_name = "SECONDS",
            default_value_t = 5.0,
        )]
        timeout: f32,

        /// Dont clean topics, only log them
        #[arg(long)]
        dry_run: bool,
    },

    /// Log values from subscribed topics to stdout
    #[command(visible_alias = "l")]
    Log {
        /// Topics to watch
        #[arg(
            env = "MQTTUI_TOPIC",
            value_hint = ValueHint::Other,
            default_value = "#",
        )]
        topic: Vec<String>,

        /// Output incoming packages as newline-delimited JSON
        #[arg(short, long)]
        json: bool,

        /// Show full MQTT communication
        #[arg(short, long)]
        verbose: bool,
    },

    /// Wait for the first message on the given topic(s) and return its payload to stdout.
    ///
    /// Returns exactly one payload of the first received message on the given topic(s).
    /// The topic of the received message is printed to stderr.
    /// This means that you can handle stdout and stderr separately.
    ///
    /// This can be helpful for scripting to get the current temperature reading and pipe it to somewhere else:
    ///
    /// `echo "The temperature is $(mqttui read-one room/temp)"`
    ///
    /// The output is the exact payload in its binary form.
    /// This might be valid ASCII / Unicode but could also be something not intended to be displayed on a terminal.
    /// For a human readable format use `--pretty` or `mqttui log`.
    #[command(visible_alias = "r", visible_alias = "read")]
    ReadOne {
        /// Topics to watch
        #[arg(
            env = "MQTTUI_TOPIC",
            value_hint = ValueHint::Other,
            default_value = "#",
        )]
        topic: Vec<String>,

        /// Do not return on a retained message on connection, wait for another message to arrive
        #[arg(long, short = 'r')]
        ignore_retained: bool,

        /// Only return the retained message, exit with 1 if there is none
        #[arg(long, short = 'R', conflicts_with = "ignore_retained")]
        only_retained: bool,

        /// Parse the payload and print it in a human readable pretty form.
        ///
        /// This might not be useful for piping the data.
        #[arg(short, long)]
        pretty: bool,
    },

    /// Publish a value quickly
    #[command(visible_alias = "p", visible_alias = "pub")]
    Publish {
        /// Topic to publish to
        #[arg(value_hint = ValueHint::Other)]
        topic: String,

        /// Payload to be published.
        ///
        /// Reads from stdin when not specified.
        /// This allows file content to be sent via pipes like this (bash):
        ///
        /// `mqttui publish some/topic </etc/hostname`
        ///
        /// `cowsay "I was here" | mqttui publish some/topic`
        #[arg(value_hint = ValueHint::Unknown)]
        payload: Option<String>,

        /// Publish the MQTT message retained
        #[arg(short, long, env = "MQTTUI_RETAIN")]
        retain: bool,

        /// Show full MQTT communication
        #[arg(short, long)]
        verbose: bool,
    },
}

#[allow(clippy::doc_markdown)]
#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommands: Option<Subcommands>,

    /// Topic to watch
    #[arg(
        env = "MQTTUI_TOPIC",
        value_hint = ValueHint::Other,
        default_value = "#",
    )]
    pub topic: Vec<String>,

    /// Truncate the payloads stored to the given size.
    ///
    /// Payloads bigger than that are truncated and not inspected for formats like JSON or MessagePack.
    /// Only their beginning up to the specified amount of bytes can be viewed.
    /// Increasing this value might result in higher memory consumption especially over time.
    #[arg(
        long,
        env = "MQTTUI_PAYLOAD_SIZE_LIMIT",
        value_hint = ValueHint::Other,
        default_value_t = 8_000,
    )]
    pub payload_size_limit: usize,

    // Keep at the end to not mix the next_help_heading with other options
    #[command(flatten, next_help_heading = "MQTT Connection")]
    pub mqtt_connection: MqttConnection,
}

/// Arguments related to the MQTT connection.
#[derive(Debug, Args)]
pub struct MqttConnection {
    /// URL which represents how to connect to the MQTT broker.
    ///
    /// Examples:
    /// `mqtt://localhost`
    /// `mqtt://localhost:1883`
    /// `mqtts://localhost`
    /// `mqtts://localhost:8883`
    /// `ws://localhost/path`
    /// `ws://localhost:9001/path`
    /// `wss://localhost/path`
    /// `wss://localhost:9001/path`
    #[arg(
        short,
        long,
        env = "MQTTUI_BROKER",
        value_hint = ValueHint::Url,
        value_name = "URL",
        global = true,
        default_value = "mqtt://localhost",
    )]
    pub broker: Broker,

    /// Username to access the mqtt broker.
    ///
    /// Anonymous access when not supplied.
    #[arg(
        short,
        long,
        env = "MQTTUI_USERNAME",
        value_hint = ValueHint::Username,
        value_name = "STRING",
        requires = "password",
        global = true,
    )]
    pub username: Option<String>,

    /// Password to access the mqtt broker.
    ///
    /// Consider using a connection with TLS to the broker.
    /// Otherwise the password will be transported in plaintext.
    ///
    /// Passing the password via command line is insecure as the password can be read from the history!
    /// You should pass it via environment variable.
    #[arg(
        long,
        env = "MQTTUI_PASSWORD",
        value_hint = ValueHint::Other,
        value_name = "STRING",
        hide_env_values = true,
        requires = "username",
        global = true,
    )]
    pub password: Option<String>,

    /// Specify the client id to connect with
    #[arg(
        short = 'i',
        long,
        env = "MQTTUI_CLIENTID",
        value_hint = ValueHint::Other,
        value_name = "STRING",
        global = true,
    )]
    pub client_id: Option<String>,

    /// Path to the TLS client certificate file.
    ///
    /// Used together with --client-private-key to enable TLS client authentication.
    /// The file has to be a DER-encoded X.509 certificate serialized to PEM.
    #[arg(
        long,
        env = "MQTTUI_CLIENT_CERTIFICATE",
        value_hint = ValueHint::FilePath,
        value_name = "FILEPATH",
        requires = "client_private_key",
        global = true,
    )]
    pub client_cert: Option<std::path::PathBuf>,

    /// Path to the TLS client private key file.
    ///
    /// Used together with --client-cert to enable TLS client authentication.
    /// The file has to be a DER-encoded ASN.1 file in PKCS#8 form serialized to PEM.
    #[arg(
        long,
        env = "MQTTUI_CLIENT_PRIVATE_KEY",
        value_hint = ValueHint::FilePath,
        value_name = "FILEPATH",
        alias = "client-key",
        requires = "client_cert",
        global = true,
    )]
    pub client_private_key: Option<std::path::PathBuf>,

    /// Allow insecure TLS connections
    #[arg(long, global = true)]
    pub insecure: bool,
}

#[derive(Debug, Clone)]
pub enum Broker {
    Tcp { host: String, port: u16 },
    Ssl { host: String, port: u16 },
    WebSocket(Url),
    WebSocketSsl(Url),
}

impl core::str::FromStr for Broker {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(input)?;
        anyhow::ensure!(url.has_host(), "Broker requires a Host");

        if matches!(url.scheme(), "mqtt" | "mqtts") {
            anyhow::ensure!(
                url.path().is_empty() || url.path() == "/",
                "TCP connections only use host (and port) but no path"
            );
        }

        if !matches!(url.scheme(), "ws" | "wss") {
            anyhow::ensure!(url.query().is_none(), "URL query is not used");
            anyhow::ensure!(url.username().is_empty(), "Use --username instead");
            anyhow::ensure!(url.password().is_none(), "Use --password instead");
        }

        anyhow::ensure!(url.port() != Some(0), "Port can not be 0");

        let broker = match url.scheme() {
            "mqtt" => Self::Tcp {
                host: url.host_str().context("Broker requires a Host")?.to_owned(),
                port: url.port().unwrap_or(1883),
            },
            "mqtts" => Self::Ssl {
                host: url.host_str().context("Broker requires a Host")?.to_owned(),
                port: url.port().unwrap_or(8883),
            },
            "ws" => Self::WebSocket(url),
            "wss" => Self::WebSocketSsl(url),
            _ => anyhow::bail!("Broker URL scheme is not supported"),
        };

        Ok(broker)
    }
}

impl core::fmt::Display for Broker {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Tcp { host, port } => {
                if *port == 1883 {
                    write!(fmt, "mqtt://{host}")
                } else {
                    write!(fmt, "mqtt://{host}:{port}")
                }
            }
            Self::Ssl { host, port } => {
                if *port == 8883 {
                    write!(fmt, "mqtts://{host}")
                } else {
                    write!(fmt, "mqtts://{host}:{port}")
                }
            }
            Self::WebSocket(url) | Self::WebSocketSsl(url) => url.fmt(fmt),
        }
    }
}

#[test]
fn verify() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
