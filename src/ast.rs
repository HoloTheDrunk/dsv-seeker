use crate::operators::*;

use crate::Rule;

use std::io::BufRead;

use {
    pest::{
        error::{Error, ErrorVariant},
        iterators::{Pair, Pairs},
    },
    regex::Regex,
};

#[derive(Debug, Default)]
pub struct Ast(Vec<Command>);

#[derive(Debug)]
pub enum Command {
    Select(Column),
    Comparison(String, Comparison),
    Enumerate(String),
}

#[derive(Debug)]
pub enum Column {
    Names(Vec<String>),
    All,
}

#[derive(Clone, Debug)]
pub enum Comparison {
    Equals(String),
    Matches(Regex),
}

macro_rules! fields {
    ($pair:ident |> $children:ident $(: $($field:ident),*)?) => {
        let mut $children = $pair.clone().into_inner();

        $($(
            let $field = $children
                .next()
                .ok_or_else(|| Error::new_from_span(
                    ErrorVariant::ParsingError {
                        positives: vec![$pair.as_rule()],
                        negatives: vec![]
                    },
                    $pair.as_span()
                ))?;
        )*)?
    };
}

pub fn build(pairs: Pairs<Rule>) -> Result<Ast, Error<Rule>> {
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

            Ok(Command::Select(Column::from(columns)))
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
                "=" => Ok(Command::Comparison(
                    lhs.as_str().to_owned(),
                    Comparison::Equals(rhs.as_str().to_owned()),
                )),
                "like" => Ok(Command::Comparison(
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
        Rule::enumerate => {
            let column = pair
                .into_inner()
                .next()
                .map(|atom| atom.as_str().to_owned())
                .unwrap();

            Ok(Command::Enumerate(column))
        }
        rule => Err(Error::new_from_span(
            ErrorVariant::CustomError {
                message: format!("Unhandled rule {rule:?}"),
            },
            pair.as_span(),
        )),
    }
}

impl From<Vec<String>> for Column {
    fn from(columns: Vec<String>) -> Self {
        if columns.len() == 1 && columns.get(0).unwrap() == "*" {
            Column::All
        } else {
            Column::Names(columns)
        }
    }
}

impl Ast {
    pub fn run_on(
        &self,
        reader: csv::Reader<Box<dyn BufRead>>,
        delim: char,
    ) -> Result<Vec<String>, String> {
        let Ast(commands) = &self;

        // let mut output = vec![remaining_headers];
        let mut records: Vec<csv::StringRecord> = reader
            .into_records()
            .enumerate()
            .filter_map(|(index, result)| match result {
                Ok(res) => Some(res),
                Err(err) => {
                    eprintln!("Error reading line {index}: {err}");
                    None
                }
            })
            .collect();

        for command in commands.iter() {
            records = match command {
                Command::Select(columns) => select::run(records.into_iter(), columns)?,
                Command::Comparison(column, comparison) => {
                    comparison::run(records.into_iter(), column.clone(), comparison.clone())?
                }
                Command::Enumerate(column) => enumerate::run(records.into_iter(), column.clone())?,
            }
        }

        Ok(records
            .into_iter()
            .map(|record| {
                record
                    .into_iter()
                    .map(String::from)
                    .collect::<Vec<String>>()
                    .join(delim.to_string().as_str())
            })
            .collect::<Vec<String>>())
    }
}
