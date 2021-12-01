extern crate clap;
extern crate trust_dns_resolver;

mod common;
mod healthcheck;

use std::net::IpAddr;
use std::str::FromStr;
use std::time;

use clap::{value_t_or_exit, App, Arg};
use trust_dns_proto::rr::record_type::RecordType;
use trust_dns_resolver::config::*;
use trust_dns_resolver::Resolver;

use healthcheck::Healthcheck;

fn main() {
    let mut app = App::new("dns-watchdog")
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
            Arg::with_name("query-type")
                .value_name("QUERY_TYPE")
                .default_value("A")
                .help("indicates what type of query is required — ANY, A, MX, SIG, etc.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("watchdog-name")
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
        );
    for a in common::cli_args().iter() {
        app = app.arg(a);
    }
    let matches = app.get_matches();

    let healthcheck = DNSHealthcheck::new(DNSHealthcheckParams {
        server: matches.value_of("server").unwrap(),
        target: matches.value_of("name").unwrap(),
        name: matches.value_of("watchdog-name").unwrap(),
        query_type: matches.value_of("query-type").unwrap(),
        timeout: value_t_or_exit!(matches.value_of("timeout"), f64),
        delay: value_t_or_exit!(matches.value_of("delay"), f64),
        attempts: value_t_or_exit!(matches.value_of("attempts"), usize),
        port: value_t_or_exit!(matches.value_of("port"), u16),
        start_script: Some(matches.value_of("start-script").unwrap_or_default()),
        stop_script: Some(matches.value_of("stop-script").unwrap_or_default()),
    });
    healthcheck.run()
}

struct DNSHealthcheckParams<'a> {
    server: &'a str,
    target: &'a str,
    name: &'a str,
    query_type: &'a str,
    timeout: f64,
    delay: f64,
    attempts: usize,
    port: u16,
    start_script: Option<&'a str>,
    stop_script: Option<&'a str>,
}

struct DNSHealthcheck<'a> {
    params: DNSHealthcheckParams<'a>,
    client: Resolver,
}

impl<'a> DNSHealthcheck<'a> {
    pub fn new(params: DNSHealthcheckParams<'a>) -> DNSHealthcheck<'a> {
        let config = ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_clear(
                &vec![params.server.parse::<IpAddr>().unwrap()],
                params.port,
                true,
            ),
        );

        let mut options = ResolverOpts::default();
        options.timeout = time::Duration::from_millis((params.timeout * 1000.) as u64);
        options.use_hosts_file = false;
        options.cache_size = 0;
        options.attempts = params.attempts;

        return DNSHealthcheck {
            client: match Resolver::new(config, options) {
                Ok(c) => c,
                Err(e) => panic!("Fail to init client: {}", e),
            },
            params: params,
        };
    }
}

impl<'a> Healthcheck for DNSHealthcheck<'a> {
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
        return match self.client.lookup(
            self.params.target,
            RecordType::from_str(self.params.query_type).unwrap(),
        ) {
            Ok(_) => true,
            Err(_) => false,
        };
    }
}
