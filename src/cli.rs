use clap::{App, AppSettings, Arg, SubCommand};

pub fn build() -> App<'static, 'static> {
    App::new("MQTT TUI")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .global_setting(AppSettings::ColoredHelp)
        .subcommand(
            SubCommand::with_name("publish")
                .about("Publish a value quickly")
                .alias("p")
                .arg(
                    Arg::with_name("Topic")
                        .value_name("TOPIC")
                        .help("Topic to publish to")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("Payload")
                        .value_name("PAYLOAD")
                        .help("Payload to be published")
                        .required(true),
                )
                .arg(
                    Arg::with_name("retain")
                        .short("r")
                        .long("retain")
                        .help("Publish the MQTT message retained"),
                )
                .arg(
                    Arg::with_name("verbose")
                        .short("v")
                        .long("verbose")
                        .help("Show full MQTT communication"),
                ),
        )
        .arg(
            Arg::with_name("Host")
                .short("h")
                .long("host")
                .value_name("HOST")
                .global(true)
                .takes_value(true)
                .help("Host on which the MQTT Broker is running")
                .default_value("localhost"),
        )
        .arg(
            Arg::with_name("Port")
                .short("p")
                .long("port")
                .value_name("INT")
                .global(true)
                .takes_value(true)
                .help("Port on which the MQTT Broker is running")
                .default_value("1883"),
        )
        .arg(
            Arg::with_name("Topic")
                .value_name("TOPIC")
                .takes_value(true)
                .default_value("#")
                .help("Topic to watch"),
        )
}
