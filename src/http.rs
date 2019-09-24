extern crate clap;
extern crate reqwest;

use clap::{value_t_or_exit, App, Arg};
use reqwest::Client;
use std::{thread, time};

fn main() {
    let matches = App::new("http-watchdog")
        .about("HTTP watchdog for exabgp")
        .arg(
            Arg::with_name("uri")
                .value_name("URI")
                .required(true)
                .help("URI to target")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("watchdog_name")
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
            Arg::with_name("check_status")
                .long("check-status")
                .help("check http status"),
        )
        .get_matches();

    run(
        matches.value_of("uri").unwrap(),
        matches.value_of("watchdog_name").unwrap(),
        value_t_or_exit!(matches.value_of("timeout"), f64),
        value_t_or_exit!(matches.value_of("delay"), f64),
        matches.is_present("check_status"),
    )
}

fn run(uri: &str, name: &str, timeout: f64, delay: f64, check_status: bool) {
    let client = match Client::builder()
        .timeout(time::Duration::from_millis((timeout * 1000.) as u64))
        .build()
    {
        Ok(c) => c,
        Err(e) => panic!("Fail to init client: {}", e),
    };

    let mut prev_check = false;
    loop {
        let ok: bool = match client.get(uri).send() {
            Ok(res) => !check_status || res.status().as_u16() < 400,
            Err(_) => false,
        };

        if ok && !prev_check {
            println!("announce watchdog {}", name);
        } else if !ok && prev_check {
            println!("withdraw watchdog {}", name);
        }
        prev_check = ok;
        thread::sleep(time::Duration::from_millis((delay * 1000.) as u64));
    }
}
