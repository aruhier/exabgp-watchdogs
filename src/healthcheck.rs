extern crate ctrlc;

use crate::common::launch_script;
use std::{process, thread, time};

pub trait Healthcheck {
    fn get_name(&self) -> &str;
    fn get_delay(&self) -> f64;
    fn get_start_script(&self) -> Option<&str>;
    fn get_stop_script(&self) -> Option<&str>;

    fn check(&mut self) -> bool;
    fn run(&mut self) {
        let mut prev_check = false;

        {
            let stop_script = self.get_stop_script().unwrap_or_default().to_owned();
            if !stop_script.is_empty() {
                ctrlc::set_handler(move || {
                    launch_script(&stop_script);
                    process::exit(0);
                })
                .expect("Error setting Ctrl-C handler");
            }
        }
        loop {
            let ok: bool = self.check();

            if ok && !prev_check {
                println!("announce watchdog {}", self.get_name());
                let start_script = self.get_start_script().unwrap_or_default();
                if !start_script.is_empty() {
                    launch_script(start_script);
                }
            } else if !ok && prev_check {
                println!("withdraw watchdog {}", self.get_name());
                let stop_script = self.get_stop_script().unwrap_or_default();
                if !stop_script.is_empty() {
                    launch_script(stop_script);
                }
            }
            prev_check = ok;
            thread::sleep(time::Duration::from_millis(
                (self.get_delay() * 1000.) as u64,
            ));
        }
    }
}
