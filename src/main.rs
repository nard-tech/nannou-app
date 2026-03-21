mod buddhabrot;
mod goldbach_comet;
mod sample;

mod mathlib;

type CommandHandler = fn();

const COMMANDS: &[(&str, CommandHandler)] = &[
    ("buddhabrot", buddhabrot::run),
    ("goldbach_comet", goldbach_comet::run),
    ("sample", sample::run),
];

fn main() {
    let mut args = std::env::args().skip(1);
    let cmd = args.next();

    match cmd.as_deref().and_then(find_command) {
        Some(run) => run(),
        None => {
            if let Some(other) = cmd.as_deref() {
                eprintln!("Unknown target: {other}");
            }
            print_usage();
            std::process::exit(1);
        }
    }
}

fn find_command(cmd: &str) -> Option<CommandHandler> {
    COMMANDS
        .iter()
        .find(|(name, _)| *name == cmd)
        .map(|(_, run)| *run)
}

fn print_usage() {
    let commands = COMMANDS
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join("|");
    eprintln!("Usage: cargo run -- <{commands}>");
}
