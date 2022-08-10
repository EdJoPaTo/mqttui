use clap::{self, Parser};

#[derive(Parser, Debug)]
pub enum SubCommands {
    /// Clean retained messages from the broker
    #[clap(
        long_about = r#"
        Clean retained messages from the broker. This works by subscribing to the topic 
        and waiting for messages with the retained flag. Then a message with an empty payload 
        is published retained which clears  the topic on the broker. Ends on the 
        first non retained message or when the timeout is reached
    "#,
        visible_alias = "c",
        visible_alias = "clean"
    )]
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
        #[clap(
            required = true,
            env = "MQTT_TOPIC",
            name = "TOPIC",
            default_value = "#"
        )]
        topics: Vec<String>,

        /// Show full MQTT communication
        #[clap(short, long)]
        verbose: bool,
    },

    #[clap(visible_alias = "p")]
    #[clap(visible_alias = "pub")]
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
pub struct Opt {
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

    /// Username to access the mqtt broker
    #[clap(long, short, name = "STRING", env = "MQTTUI_USERNAME", global = true)]
    pub username: Option<String>,

    /// Password to access the mqtt broker
    #[clap(
        long,
        short = 'x',
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

    /// Use TLS when connecting, defaults to false unless port 8883 is specified
    #[clap(
        long,
        short,
        value_name = "BOOL",
        env = "MQTTUI_ENCRYPTION",
        global = true
    )]
    pub encryption: Option<bool>,

    /// The Topic to subscribe to when connecting to the broker
    #[clap(value_name = "TOPIC", env = "MQTTUI_TOPIC", default_value = "#")]
    pub topic: String,

    /// Allow connections to insecure or untrusted servers, skips the server verification, use with caution
    #[clap(long, global = true)]
    pub insecure: bool,
}

#[test]
fn verify() {
    use clap::CommandFactory;
    Opt::command().debug_assert();
}
