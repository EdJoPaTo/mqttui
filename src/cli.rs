use clap::{Parser, ValueHint};

#[derive(Debug, Parser)]
pub enum SubCommands {
    /// Clean retained messages from the broker.
    ///
    /// This works by subscribing to the topic and waiting for messages with the retained flag.
    /// Then a message with an empty payload is published retained which clears the topic on the broker.
    /// Ends on the first non retained message or when the timeout is reached.
    #[clap(visible_alias = "c", visible_alias = "clean")]
    CleanRetained {
        /// Topic which gets cleaned.
        ///
        /// Supports filters like 'foo/bar/#'.
        #[clap(value_hint = ValueHint::Other)]
        topic: String,

        /// When there is no message received for the given time the operation is considered done
        #[clap(
            long,
            value_hint = ValueHint::Other,
            value_name = "SECONDS",
            default_value = "5",
        )]
        timeout: f32,

        /// Dont clean topics, only log them
        #[clap(long)]
        dry_run: bool,
    },

    /// Log values from subscribed topics to stdout
    #[clap(visible_alias = "l")]
    Log {
        /// Topics to watch
        #[clap(
            env = "MQTTUI_TOPIC",
            value_hint = ValueHint::Other,
            default_value = "#",
        )]
        topic: Vec<String>,

        /// Show full MQTT communication
        #[clap(short, long)]
        verbose: bool,
    },

    /// Publish a value quickly
    #[clap(visible_alias = "p", visible_alias = "pub")]
    Publish {
        /// Topic to publish to
        #[clap(value_hint = ValueHint::Other)]
        topic: String,

        /// Payload to be published
        #[clap(value_hint = ValueHint::Unknown)]
        payload: String,

        /// Publish the MQTT message retained
        #[clap(short, long, env = "MQTTUI_RETAIN")]
        retain: bool,

        /// Show full MQTT communication
        #[clap(short, long)]
        verbose: bool,
    },
}

#[derive(Debug, Parser)]
#[clap(about, author, version)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommands: Option<SubCommands>,

    /// Host on which the MQTT Broker is running
    #[clap(
        short,
        long,
        env = "MQTTUI_BROKER",
        value_hint = ValueHint::Hostname,
        value_name = "HOST",
        global = true,
        default_value = "localhost",
    )]
    pub broker: String,

    /// Host on which the MQTT Broker is running
    #[clap(
        short,
        long,
        env = "MQTTUI_PORT",
        value_hint = ValueHint::Other,
        value_name = "INT",
        global = true,
        default_value = "1883",
    )]
    pub port: u16,

    /// Username to access the mqtt broker.
    ///
    /// Anonymous access when not supplied.
    #[clap(
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
    /// Passing the password via command line is insecure as the password can be read from the history!
    #[clap(
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
    #[clap(
        short = 'i',
        long,
        env = "MQTTUI_CLIENTID",
        value_hint = ValueHint::Other,
        value_name = "STRING",
        global = true,
    )]
    pub client_id: Option<String>,

    /// Topic to watch
    #[clap(
        env = "MQTTUI_TOPIC",
        value_hint = ValueHint::Other,
        default_value = "#",
    )]
    pub topic: String,
}

#[test]
fn verify() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
