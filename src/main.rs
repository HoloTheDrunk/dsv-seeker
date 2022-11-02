use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use {
    clap::Parser as ClapParser,
    pest::{
        error::{Error, ErrorVariant},
        iterators::{Pair, Pairs},
        Parser,
    },
    regex::Regex,
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

#[derive(Debug, Default)]
struct Ast(Vec<Command>);

#[derive(Debug)]
enum Command {
    Select(Column),
    Where(String, Comparison),
}

#[derive(Debug)]
enum Column {
    Names(Vec<String>),
    All,
}

#[derive(Debug)]
enum Comparison {
    Equals(String),
    Matches(Regex),
}

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

macro_rules! fields {
    ($pair:ident |> $children:ident $(: $($field:ident),*)?) => {
        let mut $children = $pair.clone().into_inner();

        $(
            $(
                let $field = $children
                    .next()
                    .ok_or_else(|| Error::new_from_span(
                        ErrorVariant::ParsingError {
                            positives: vec![$pair.as_rule()],
                            negatives: vec![]
                        },
                        $pair.as_span()
                    ))?;
            )*
        )?
    };
}

fn build_ast(pairs: Pairs<Rule>) -> Result<Ast, Error<Rule>> {
    let mut commands = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::EOI => break,
            _ => commands.push(build_command(pair)?),
        }
    }

    Ok(Ast(commands))
}

fn build_command(pair: Pair<Rule>) -> Result<Command, Error<Rule>> {
    match pair.as_rule() {
        Rule::select => {
            let columns = pair
                .into_inner()
                .map(|quoted| quoted.as_str().to_owned())
                .collect::<Vec<String>>();

            Ok(Command::Select(
                if columns.len() == 1 && columns.get(0).unwrap() == "*" {
                    Column::All
                } else {
                    Column::Names(columns)
                },
            ))
        }
        Rule::comparison => {
            fields!(pair |> children : lhs, comparator, rhs);

            let matchexpr = Regex::new(rhs.as_str()).map_err(|err| {
                Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!(
                            "Invalid regex {}",
                            match err {
                                regex::Error::Syntax(message) => format!("'{rhs}': {message}"),
                                _ => format!("'{rhs}'"),
                            }
                        ),
                    },
                    pair.as_span(),
                )
            })?;

            match comparator.as_str() {
                "=" => Ok(Command::Where(
                    lhs.as_str().to_owned(),
                    Comparison::Equals(rhs.as_str().to_owned()),
                )),
                "like" => Ok(Command::Where(
                    lhs.as_str().to_owned(),
                    Comparison::Matches(matchexpr),
                )),
                invalid => Err(Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("Unhandled comparator '{invalid}'"),
                    },
                    pair.as_span(),
                )),
            }
        }
        rule => Err(Error::new_from_span(
            ErrorVariant::CustomError {
                message: format!("Unhandled rule {rule:?}"),
            },
            pair.as_span(),
        )),
    }
}

impl Ast {
    fn run_on(
        &self,
        mut reader: csv::Reader<Box<dyn BufRead>>,
        delim: char,
    ) -> Result<Vec<String>, String> {
        let Ast(commands) = &self;

        let mut header_record = reader.headers().map_err(|err| format!("{err:?}"))?.clone();

        let headers = header_record
            .iter()
            .enumerate()
            .map(|(k, v)| (v.to_owned(), k))
            .collect::<HashMap<String, usize>>();

        for command in commands {
            if let Command::Select(columns) = command {
                header_record = apply_select(columns, &headers, header_record)?;
            }
        }

        let remaining_headers = header_record
            .iter()
            .map(String::from)
            .collect::<Vec<String>>()
            .join(delim.to_string().as_str());

        let mut output = vec![remaining_headers];

        for record in reader.records() {
            match record {
                Ok(record) => {
                    let mut record = record;
                    let mut rejected = false;

                    for command in commands.iter() {
                        match command {
                            Command::Select(columns) => {
                                record = apply_select(columns, &headers, record)?
                            }
                            Command::Where(column, comparison) => {
                                let value = record
                                    .get(
                                        *headers
                                            .get(column.as_str())
                                            .ok_or(format!("Invalid column '{column}'"))?,
                                    )
                                    .ok_or(format!("Missing column {column}"))?;
                                match comparison {
                                    Comparison::Equals(other) => {
                                        if value != other {
                                            rejected = true;
                                            break;
                                        }
                                    }
                                    Comparison::Matches(pattern) => {
                                        if !pattern.is_match(value) {
                                            rejected = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !rejected {
                        output.push(
                            record
                                .iter()
                                .map(String::from)
                                .collect::<Vec<String>>()
                                .join(delim.to_string().as_ref()),
                        );
                    }
                }
                Err(err) => return Err(format!("DSV parsing failed: {err}")),
            }
        }

        Ok(output)
    }
}

fn apply_select(
    columns: &Column,
    headers: &HashMap<String, usize>,
    record: csv::StringRecord,
) -> Result<csv::StringRecord, String> {
    if let Column::Names(names) = columns {
        let mut to_keep = Vec::new();

        for name in names {
            to_keep.push(
                headers
                    .get(name.as_str())
                    .ok_or(format!("Invalid column '{name}'"))?,
            );
        }

        Ok(csv::StringRecord::from(
            to_keep
                .iter()
                .map(|&&index| record.get(index).ok_or(format!("Missing column {index}")))
                .collect::<Result<Vec<&str>, String>>()?,
        ))
    } else {
        Ok(record)
    }
}
