#[derive(Default)]
pub struct CliOptions {
    pub log_file: Option<String>,
    pub tor_dir: Option<String>,
}

pub fn get_args_from_cli() -> std::env::Args {
    std::env::args()
}

pub fn get_cli_options(args: impl Iterator<Item=String>) -> CliOptions {
    let mut opts: CliOptions = Default::default();
    for arg in args {
        if arg.starts_with("--log_file=") || arg.starts_with("--log-file=") || arg.starts_with("-l=") {
            opts.log_file = get_arg_val(&arg);
        }
        if arg.starts_with("--tor_dir=") || arg.starts_with("--tor-dir=") || arg.starts_with("-t=") {
            opts.tor_dir = get_arg_val(&arg);
        }
    }
    opts
}

#[allow(dead_code)]
pub fn get_args_from_string(s: &str) -> impl Iterator<Item=String> + '_ {
    s.split(char::is_whitespace).map(|v| v.to_string())
}

fn get_arg_val(arg: &str) -> Option<String> {
    let parts: Vec<&str> = arg.split('=').collect();
    if let Some(val) = parts.get(1) {
        return Some(val.to_string());
    }
    None
}