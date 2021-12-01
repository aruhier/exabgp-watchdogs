extern crate clap;
extern crate reqwest;

mod common;
mod healthcheck;

use clap::{value_t_or_exit, App, Arg};
use reqwest::blocking::Client;
use std::time;

use healthcheck::Healthcheck;

fn main() {
    let mut app = App::new("http-watchdog")
        .about("HTTP watchdog for exabgp")
        .arg(
            Arg::with_name("uri")
                .value_name("URI")
                .required(true)
                .help("URI to target")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("watchdog-name")
                .short("n")
                .long("name")
                .value_name("WATCHDOG_NAME")
                .help("watchdog name to print in messages")
                .takes_value(true)
                .default_value("http"),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .value_name("SECONDS")
                .help("timeout for the HTTP client")
                .takes_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::with_name("delay")
                .long("delay")
                .value_name("SECONDS")
                .help("delay between tests")
                .takes_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::with_name("check-status")
                .long("check-status")
                .help("check http status"),
        );
    for a in common::cli_args().iter() {
        app = app.arg(a);
    }
    let matches = app.get_matches();

    let mut uri = matches.value_of("uri").unwrap().to_owned();
    if !uri.starts_with("http://") && !uri.starts_with("https://") {
        uri = format!("http://{}", uri);
    }
    let healthcheck = HTTPHealthcheck::new(HTTPHealthcheckParams {
        uri: &uri,
        name: matches.value_of("watchdog-name").unwrap(),
        timeout: value_t_or_exit!(matches.value_of("timeout"), f64),
        delay: value_t_or_exit!(matches.value_of("delay"), f64),
        check_status: matches.is_present("check-status"),
        start_script: Some(matches.value_of("start-script").unwrap_or_default()),
        stop_script: Some(matches.value_of("stop-script").unwrap_or_default()),
    });
    healthcheck.run()
}

struct HTTPHealthcheckParams<'a> {
    uri: &'a str,
    name: &'a str,
    timeout: f64,
    delay: f64,
    check_status: bool,
    start_script: Option<&'a str>,
    stop_script: Option<&'a str>,
}

struct HTTPHealthcheck<'a> {
    params: HTTPHealthcheckParams<'a>,
    client: Client,
}

impl<'a> HTTPHealthcheck<'a> {
    pub fn new(params: HTTPHealthcheckParams<'a>) -> HTTPHealthcheck<'a> {
        return HTTPHealthcheck {
            client: match Client::builder()
                .timeout(time::Duration::from_millis((params.timeout * 1000.) as u64))
                .build()
            {
                Ok(c) => c,
                Err(e) => panic!("Fail to init client: {}", e),
            },
            params: params,
        };
    }
}

impl<'a> Healthcheck for HTTPHealthcheck<'a> {
    fn get_name(&self) -> &str {
        return self.params.name;
    }
    fn get_delay(&self) -> f64 {
        return self.params.delay;
    }
    fn get_start_script(&self) -> Option<&str> {
        return self.params.start_script;
    }
    fn get_stop_script(&self) -> Option<&str> {
        return self.params.stop_script;
    }

    fn check(&self) -> bool {
        return match self.client.get(self.params.uri).send() {
            Ok(res) => !self.params.check_status || res.status().as_u16() < 400,
            Err(err) => {
                eprintln!("Failure {}", err);
                false
            }
        };
    }
}
