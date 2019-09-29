use clap::Arg;
use std::process;

pub fn launch_script(script: &str) {
    match process::Command::new(script)
        .stdout(process::Stdio::null())
        .output()
    {
        Ok(_) => return,
        Err(err) => eprintln!("Error when launching \"{}\": {}", script, err),
    }
}

pub fn cli_args<'a>() -> [Arg<'a, 'a>; 2] {
    return [
        Arg::with_name("start_script")
            .long("start_script")
            .value_name("PATH")
            .help("script to trigger when announcement starts.")
            .takes_value(true),
        Arg::with_name("stop_script")
            .long("stop_script")
            .value_name("PATH")
            .help("script to trigger when announcement stops.")
            .takes_value(true),
    ];
}
