use clap::{App, AppSettings, Arg};

#[derive(Debug)]
pub struct RuntimeArguments {
    pub verbose: bool,
    pub host: String,
    pub port: u16,
    pub topic: String,
    pub value: Option<String>,
}

pub fn build_cli() -> App<'static, 'static> {
    App::new("Quick MQTT CLI")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::ColoredHelp)
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Show full MQTT communication"),
        )
        .arg(
            Arg::with_name("Host")
                .short("h")
                .long("host")
                .value_name("HOST")
                .takes_value(true)
                .help("Host on which the MQTT Broker is running")
                .default_value("localhost"),
        )
        .arg(
            Arg::with_name("Port")
                .short("p")
                .long("port")
                .value_name("INT")
                .takes_value(true)
                .help("Port on which the MQTT Broker is running")
                .default_value("1883"),
        )
        .arg(
            Arg::with_name("Topic")
                .value_name("TOPIC")
                .takes_value(true)
                .default_value("#")
                .help("Topic to watch or publish to"),
        )
        .arg(
            Arg::with_name("Payload")
                .value_name("PAYLOAD")
                .help("(optional) Payload to be published. If none is given it is instead subscribed to the topic."),
        )
}

pub fn get_runtime_arguments() -> RuntimeArguments {
    let matches = build_cli().get_matches();

    let verbose = matches.is_present("verbose");

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
        .expect("MQTT Base Topic could not be read from command line")
        .to_owned();

    let value = matches.value_of("Payload").map(|o| o.to_owned());

    RuntimeArguments {
        verbose,
        host,
        port,
        topic,
        value,
    }
}
