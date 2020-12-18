use clap::{App, AppSettings, Arg, SubCommand};

#[derive(Debug)]
pub struct RuntimeArguments {
    pub verbose: bool,
    pub host: String,
    pub port: u16,
    pub topic: String,
    pub payload: Option<String>,
}

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

pub fn get_runtime_arguments() -> RuntimeArguments {
    let main_matches = build().get_matches();
    let publish_matches = main_matches.subcommand_matches("publish");
    let matches = publish_matches.unwrap_or(&main_matches);

    let host = matches
        .value_of("Host")
        .expect("Host could not be read from command line")
        .to_owned();

    let port = matches
        .value_of("Port")
        .and_then(|s| s.parse::<u16>().ok())
        .expect("MQTT Server Port could not be read from command line.");

    let topic = matches
        .value_of("Topic")
        .expect("Topic could not be read from command line")
        .to_owned();

    let verbose = publish_matches.map_or(false, |matches| matches.is_present("verbose"));

    let payload = publish_matches
        .and_then(|matches| matches.value_of("Payload"))
        .map(std::borrow::ToOwned::to_owned);

    RuntimeArguments {
        verbose,
        host,
        port,
        topic,
        payload,
    }
}
