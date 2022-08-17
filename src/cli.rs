use clap::{self, Parser};

#[derive(Parser, Debug)]
pub enum SubCommands {
    /// Clean retained messages from the broker
    ///
    /// Clean retained messages from the broker. This works by subscribing to the topic
    /// and waiting for messages with the retained flag. Then a message with an empty payload
    /// is published retained which clears  the topic on the broker. Ends on the
    /// first non retained message or when the timeout is reached
    #[clap(visible_alias = "c", visible_alias = "clean")]
    CleanRetained {
        /// Topic which gets cleaned, Supports filters like 'foo/bar/#'
        #[clap(required = true)]
        topic: String,
        /// When there is no message received for the given time the operation is considered done
        #[clap(long, default_value = "5")]
        timeout: f32,
        // Dont clean topics, only log them
        #[clap(long)]
        dry_run: bool,
    },
    /// Log values from subscribed topics to stdout
    #[clap(visible_alias = "l")]
    Log {
        /// Topics to log
        #[clap(required = true, env = "MQTT_TOPIC", default_value = "#")]
        topic: Vec<String>,

        /// Show full MQTT communication
        #[clap(short, long)]
        verbose: bool,
    },

    #[clap(visible_alias = "p", visible_alias = "pub")]
    Publish {
        /// Topic to publish to,
        #[clap(required = true)]
        topic: String,

        /// Payload to be published
        #[clap(required = true, name = "PAYLOAD")]
        payload: String,

        /// Publish the MQTT message retained
        #[clap(short, long)]
        retain: bool,

        /// Show full MQTT communication
        #[clap(short, long)]
        verbose: bool,
    },
}

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommands: Option<SubCommands>,

    ///Host on which the MQTT Broker is running
    #[clap(
        long,
        short,
        value_name = "HOST",
        env = "MQTTUI_BROKER",
        default_value = "localhost",
        global = true
    )]
    pub broker: String,

    /// Host on which the MQTT Broker is running
    #[clap(
        long,
        short,
        value_name = "INT",
        env = "MQTTUI_PORT",
        default_value = "1883",
        global = true
    )]
    pub port: u16,

    /// Username to access the mqtt broker.
    ///
    /// Anonymous access when not supplied.
    #[clap(long, short, name = "STRING", env = "MQTTUI_USERNAME", global = true)]
    pub username: Option<String>,

    /// Password to access the mqtt broker.
    ///
    /// Passing the password via command line is insecure as the password can be read from the history!
    #[clap(
        long,
        value_name = "STRING",
        env = "MQTTUI_PASSWORD",
        global = true
    )]
    pub password: Option<String>,

    /// Specify the client id to connect with
    #[clap(
        long,
        short = 'i',
        value_name = "STRING",
        env = "MQTTUI_CLIENTID",
        global = true
    )]
    pub client_id: Option<String>,
    /// Password to access the mqtt broker
    #[clap(value_name = "TOPIC", env = "MQTTUI_PASSWORD", default_value = "#")]
    pub topic: String,
}

#[test]
fn verify() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
