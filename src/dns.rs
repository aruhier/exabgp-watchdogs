extern crate clap;
extern crate trust_dns_proto;
extern crate trust_dns_resolver;

use std::net::IpAddr;
use std::str::FromStr;
use std::{thread, time};

use clap::{value_t_or_exit, App, Arg};
use trust_dns_proto::rr::record_type::RecordType;
use trust_dns_resolver::config::*;
use trust_dns_resolver::Resolver;

fn main() {
    let matches = App::new("dns-watchdog")
        .about("DNS watchdog for exabgp")
        .arg(
            Arg::with_name("server")
                .value_name("SERVER")
                .help("dns server to test.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("name")
                .value_name("NAME")
                .help("resource record to look up.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("query_type")
                .value_name("QUERY_TYPE")
                .default_value("A")
                .help("indicates what type of query is required — ANY, A, MX, SIG, etc.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("watchdog_name")
                .short("n")
                .long("name")
                .value_name("WATCHDOG_NAME")
                .help("watchdog name to print in messages.")
                .takes_value(true)
                .default_value("dns"),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .value_name("SECONDS")
                .help("timeout for the DNS client.")
                .takes_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::with_name("attempts")
                .long("attempts")
                .value_name("ATTEMPTS")
                .help("number of attempts after considering that the DNS server is down.")
                .takes_value(true)
                .default_value("2"),
        )
        .arg(
            Arg::with_name("delay")
                .long("delay")
                .value_name("SECONDS")
                .help("delay between tests.")
                .takes_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .value_name("PORT")
                .help("dns server port.")
                .takes_value(true)
                .default_value("53"),
        )
        .get_matches();

    run(
        matches.value_of("server").unwrap(),
        matches.value_of("name").unwrap(),
        matches.value_of("query_type").unwrap(),
        value_t_or_exit!(matches.value_of("timeout"), f64),
        value_t_or_exit!(matches.value_of("delay"), f64),
        value_t_or_exit!(matches.value_of("attempts"), usize),
        value_t_or_exit!(matches.value_of("port"), u16),
    )
}

fn run(
    server: &str,
    name: &str,
    query_type: &str,
    timeout: f64,
    delay: f64,
    attempts: usize,
    port: u16,
) {
    let config = ResolverConfig::from_parts(
        None,
        vec![],
        NameServerConfigGroup::from_ips_clear(&vec![server.parse::<IpAddr>().unwrap()], port),
    );

    let mut options = ResolverOpts::default();
    options.timeout = time::Duration::from_millis((timeout * 1000.) as u64);
    options.use_hosts_file = false;
    options.cache_size = 0;
    options.attempts = attempts;

    let client = match Resolver::new(config, options) {
        Ok(c) => c,
        Err(e) => panic!("Fail to init client: {}", e),
    };

    let mut prev_check = false;
    loop {
        let ok: bool = match client.lookup(name, RecordType::from_str(query_type).unwrap()) {
            Ok(_) => true,
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
