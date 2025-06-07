use std::env;
use std::process::exit;

use clap::Parser;

#[derive(Parser)]
#[clap(author, about, version)]
enum RustyHookOpts {
    /// Initialize rusty-hook's git hooks in the current directory.
    #[clap(author, version)]
    Init {
        #[clap(long)]
        skip_hook_list: Option<String>,
    },
    /// Print the current version of rusty-hook.
    #[clap(author, version, alias = "-v")]
    Version,
    /// Run a git hook using the current directory's configuration.
    /// Ran automatically by rusty-hook's git hooks.
    #[clap(author, version)]
    Run {
        #[clap(long)]
        hook: String,
        #[clap(name = "git args", raw(true))]
        args: Option<String>,
    },
}

fn init(skip_hook_list: Option<String>) {
    if ci_info::is_ci() {
        println!("[rusty-hook] CI Environment detected. Skipping hook install");
        exit(0);
    }

    let skip_hook_list = skip_hook_list
        .as_deref()
        .map_or(vec![], |s| s.split(',').collect());

    if let Err(err) = rusty_hook::init(
        nias::get_command_runner(),
        nias::get_file_writer(),
        nias::get_file_existence_checker(),
        skip_hook_list,
    ) {
        eprintln!(
            "[rusty-hook] Fatal error encountered during initialization. Details: {}",
            err
        );
        exit(1);
    };
}

fn run(hook: String, args: Option<String>) {
    if let Err(err) = rusty_hook::run(
        nias::get_command_runner(),
        nias::get_file_existence_checker(),
        nias::get_file_reader(),
        nias::get_conditional_logger(),
        &hook,
        args,
    ) {
        match err {
            Some(e) if e == rusty_hook::NO_CONFIG_FILE_FOUND => {
                exit(rusty_hook::NO_CONFIG_FILE_FOUND_ERROR_CODE);
            }
            Some(e) => {
                eprintln!("[rusty-hook] {}", e);
                exit(1);
            }
            None => exit(1),
        }
    }
}

fn main() {
    let opts = RustyHookOpts::parse();
    match opts {
        RustyHookOpts::Init { skip_hook_list } => init(skip_hook_list),
        RustyHookOpts::Version => println!(env!("CARGO_PKG_VERSION")),
        RustyHookOpts::Run { hook, args } => run(hook, args),
    }
}
