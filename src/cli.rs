use clap::{Parser, ValueHint};
use url::Url;

#[derive(Debug, Parser)]
pub enum SubCommands {
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
    /// When the payload is parsable JSON its printed as single line non-pretty JSON.
    /// When the payload is not unicode the process exists without printing anything to stdout and with exit code 1.
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
    },

    /// Publish a value quickly
    #[command(visible_alias = "p", visible_alias = "pub")]
    Publish {
        /// Topic to publish to
        #[arg(value_hint = ValueHint::Other)]
        topic: String,

        /// Payload to be published
        #[arg(value_hint = ValueHint::Unknown)]
        payload: String,

        /// Publish the MQTT message retained
        #[arg(short, long, env = "MQTTUI_RETAIN")]
        retain: bool,

        /// Show full MQTT communication
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommands: Option<SubCommands>,

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
    /// Used together with --client-key to enable TLS client authentication.
    /// The file has to be a DER-encoded X.509 certificate serialized to PEM.
    #[arg(
        long,
        env = "MQTTUI_CLIENT_CERTIFICATE",
        value_hint = ValueHint::FilePath,
        value_name = "FILEPATH",
        requires = "client_key",
        global = true,
    )]
    pub client_cert: Option<std::path::PathBuf>,

    /// Path to the TLS client private key.
    ///
    /// Used together with --client-cert to enable TLS client authentication.
    /// The file has to be a DER-encoded ASN.1 file in PKCS#8 form serialized to PEM.
    #[arg(
        long,
        env = "MQTTUI_CLIENT_PRIVATE_KEY",
        value_hint = ValueHint::FilePath,
        value_name = "FILEPATH",
        requires = "client_cert",
        global = true,
    )]
    pub client_key: Option<std::path::PathBuf>,

    /// Allow insecure TLS connections
    #[arg(long, global = true)]
    pub insecure: bool,

    /// Topic to watch
    #[arg(
        env = "MQTTUI_TOPIC",
        value_hint = ValueHint::Other,
        default_value = "#",
    )]
    pub topic: Vec<String>,
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
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(s)?;
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

        let broker = match url.scheme() {
            "mqtt" => Self::Tcp {
                host: url
                    .host_str()
                    .ok_or_else(|| anyhow::anyhow!("Broker requires a Host"))?
                    .to_owned(),
                port: url.port().unwrap_or(1883),
            },
            "mqtts" => Self::Ssl {
                host: url
                    .host_str()
                    .ok_or_else(|| anyhow::anyhow!("Broker requires a Host"))?
                    .to_owned(),
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Tcp { host, port } => {
                if *port == 1883 {
                    f.write_str("mqtt://")?;
                    f.write_str(host)
                } else {
                    f.write_fmt(format_args!("mqtt://{host}@{port}"))
                }
            }
            Self::Ssl { host, port } => {
                if *port == 8883 {
                    f.write_str("mqtts://")?;
                    f.write_str(host)
                } else {
                    f.write_fmt(format_args!("mqtts://{host}@{port}"))
                }
            }
            Self::WebSocket(url) | Self::WebSocketSsl(url) => f.write_str(url.as_str()),
        }
    }
}

#[test]
fn verify() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
