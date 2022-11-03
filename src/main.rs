pub mod operators;
pub mod ast;

use ast::build_ast;

use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
};

use {
    clap::Parser as ClapParser,
    pest::Parser,
};

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(ClapParser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path of the desired DSV, omit for stdin
    #[arg(short, long, value_name = "PATH")]
    path: Option<PathBuf>,

    /// Delimiter, omit to use ASCII standard delimiters (0x1E and 0x1F)
    #[arg(short, long, value_name = "DELIMITER")]
    delimiter: Option<char>,

    /// Commands to run, in DSVQL
    commands: String,
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct DsvqlParser;

fn main() -> Result<(), ()> {
    let mut cli = Cli::parse();

    let input: Box<dyn BufRead> = if let Some(path) = cli.path {
        if !path.is_file() {
            panic!("{} is not a file.", path.as_path().to_string_lossy());
        }

        Box::new(BufReader::new(std::fs::File::open(path).unwrap()))
    } else {
        Box::new(BufReader::new(std::io::stdin()))
    };

    if cli.delimiter.is_none() {
        cli.delimiter = Some(0x1f as char);
    }

    let reader = csv::ReaderBuilder::new()
        .delimiter(cli.delimiter.unwrap() as u8)
        .from_reader(input);

    macro_rules! error {
        ($string:literal, $err:expr) => {{
            eprintln!(concat!("\x1b[31m", $string, ": {}\x1b[0m"), $err);
            return Err(());
        }};
    }

    match DsvqlParser::parse(Rule::program, cli.commands.as_ref()) {
        Ok(pairs) => match build_ast(pairs) {
            Ok(ast) => match ast.run_on(reader, cli.delimiter.unwrap()) {
                Ok(lines) => println!("{}", lines.join("\n")),
                Err(err) => error!("Error running query", err),
            },
            Err(err) => error!("Error building query", err),
        },
        Err(err) => error!("Error parsing query", err),
    }

    Ok(())
}
