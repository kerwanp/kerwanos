use kernel::ExitCode;

pub fn run(cmd: &str) -> &str {
    match cmd {
        "hello" => hello_cmd(),
        "shutdown" => shutdown_cmd(),
        _ => "Command not found",
    }
}

fn hello_cmd<'a>() -> &'a str {
    "Hello world"
}

fn shutdown_cmd<'a>() -> &'a str {
    kernel::exit(ExitCode::Success);
}
