use clap::{arg, Command};

use leoscript_lib::run_script;

fn cli() -> Command {
    Command::new("git")
        .about("Leo Script CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("run")
                .about("Run the script")
                .arg(arg!(<REMOTE> "The remote to clone"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("compile")
                .about("Compile the script")
                .arg(arg!(<REMOTE> "The remote to clone"))
                .arg_required_else_help(true),
        )
}

fn main() {

    //let matches = cli().get_matches();

    let output = run_script(include_str!("example.leo"), "main", None);

    match output {
        Ok(v) => println!("Script result: {:?}", v),
        Err(e) => println!("Script error: {:?}", e),
    }

}
