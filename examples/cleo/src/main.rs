use clap::{arg, Command};

use leoscript::{run_script_from_file, run_script_from_string};

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

    //let matches = cleo().get_matches();

    let file_path = "examples/cleo/src/example.leo";
    let output = run_script_from_file(file_path, "main", None);

    match output {
        Ok(v) => println!("Script result: {:#?}", v),
        Err(e) => println!("Script error: {:#?}", e),
    }

}
