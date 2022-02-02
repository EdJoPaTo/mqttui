use clap::{app_from_crate, App, Arg, ValueHint};

#[must_use]
pub fn build() -> App<'static> {
    app_from_crate!()
        .name("MQTT TUI")
        .subcommand(
            App::new("publish")
                .about("Publish a value quickly")
                .visible_aliases(&["p", "pub"])
                .arg(
                    Arg::new("Topic")
                        .value_hint(ValueHint::Other)
                        .value_name("TOPIC")
                        .takes_value(true)
                        .required(true)
                        .help("Topic to publish to")
                )
                .arg(
                    Arg::new("Payload")
                        .value_hint(ValueHint::Unknown)
                        .value_name("PAYLOAD")
                        .takes_value(true)
                        .required(true)
                        .help("Payload to be published"),
                )
                .arg(
                    Arg::new("retain")
                        .short('r')
                        .long("retain")
                        .env("MQTTUI_RETAIN")
                        .help("Publish the MQTT message retained"),
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Show full MQTT communication"),
                ),
        )
        .arg(
            Arg::new("Host")
                .short('b')
                .long("broker")
                .env("MQTTUI_BROKER")
                .value_hint(ValueHint::Hostname)
                .value_name("HOST")
                .global(true)
                .takes_value(true)
                .help("Host on which the MQTT Broker is running")
                .default_value("localhost"),
        )
        .arg(
            Arg::new("Port")
                .short('p')
                .long("port")
                .env("MQTTUI_PORT")
                .value_hint(ValueHint::Other)
                .value_name("INT")
                .global(true)
                .takes_value(true)
                .help("Port on which the MQTT Broker is running")
                .default_value("1883"),
        )
        .arg(
            Arg::new("Username")
                .short('u')
                .long("username")
                .env("MQTTUI_USERNAME")
                .value_hint(ValueHint::Username)
                .value_name("STRING")
                .global(true)
                .takes_value(true)
                .requires("Password")
                .help("Username to access the mqtt broker")
                .long_help(
                    "Username to access the mqtt broker. Anonymous access when not supplied.",
                ),
        )
        .arg(
            Arg::new("Password")
                .long("password")
                .env("MQTTUI_PASSWORD")
                .value_hint(ValueHint::Other)
                .value_name("STRING")
                .global(true)
                .takes_value(true)
                .requires("Username")
                .help("Password to access the mqtt broker")
                .long_help(
                    "Password to access the mqtt broker. Passing the password via command line is insecure as the password can be read from the history!",
                ),
        )
        .arg(
            Arg::new("Topic")
                .env("MQTTUI_TOPIC")
                .value_hint(ValueHint::Other)
                .value_name("TOPIC")
                .takes_value(true)
                .default_value("#")
                .help("Topic to watch"),
        )
}

#[test]
fn verify_app() {
    build().debug_assert();
}
