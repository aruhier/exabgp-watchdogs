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
        Arg::with_name("start-script")
            .long("start-script")
            .value_name("PATH")
            .help("script to trigger when announcement starts.")
            .takes_value(true),
        Arg::with_name("stop-script")
            .long("stop-script")
            .value_name("PATH")
            .help("script to trigger when announcement stops.")
            .takes_value(true),
    ];
}
