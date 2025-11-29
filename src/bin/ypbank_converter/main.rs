use std::{fmt, io};

use clap::Parser;
use std::fs;
use ypbank_parser::{
    dump_as_bin, dump_as_csv, dump_as_text, error, parse_from_bin, parse_from_csv, parse_from_text,
    types,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Input file path
    #[arg(long, required = true)]
    input_file: String,

    /// Input file type: text/csv/bin
    #[arg(long, required = true)]
    input_format: String,

    /// Output file type: text/csv/bin
    #[arg(long, required = true)]
    output_format: String,
}

enum Type {
    Bin,
    Csv,
    Text,
}

#[derive(Debug)]
enum Error {
    Parse(String),
    Dump(String),
    Usage(String),
    IO,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(msg) | Self::Dump(msg) | Self::Usage(msg) => {
                write!(f, "{}", msg)
            }
            Self::IO => write!(f, "IO error"),
        }
    }
}

impl From<error::ParseError> for Error {
    fn from(value: error::ParseError) -> Self {
        match value {
            error::ParseError::IOError(str) => Error::Parse(str),
            error::ParseError::InvalidFormat(err) => Error::Parse(err.to_string()),
        }
    }
}

impl From<error::DumpError> for Error {
    fn from(value: error::DumpError) -> Self {
        match value {
            error::DumpError::InternalError => Error::Dump("internal dump error".to_string()),
            error::DumpError::OutputError => Error::Dump("dump error".to_string()),
        }
    }
}

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Self {
        Error::IO
    }
}

fn parse_format(f: &str) -> Result<Type, Error> {
    match f {
        "text" => Ok(Type::Text),
        "csv" => Ok(Type::Csv),
        "bin" => Ok(Type::Bin),
        _ => Err(Error::Usage("invalid format".to_string())),
    }
}

fn parse_tx(
    reader: &mut impl io::Read,
    input_type: Type,
) -> Result<Vec<types::Transaction>, Error> {
    match input_type {
        Type::Csv => Ok(parse_from_csv(reader)?),
        Type::Text => Ok(parse_from_text(reader)?),
        Type::Bin => Ok(parse_from_bin(reader)?),
    }
}

fn dump_tx(
    writer: &mut impl io::Write,
    output_type: Type,
    transactions: &[types::Transaction],
) -> Result<(), Error> {
    match output_type {
        Type::Csv => Ok(dump_as_csv(writer, transactions)?),
        Type::Text => Ok(dump_as_text(writer, transactions)?),
        Type::Bin => Ok(dump_as_bin(writer, transactions)?),
    }
}

fn main() {
    let args = Args::parse();

    let input = fs::File::open(&args.input_file);
    let Ok(mut input_file) = input else {
        eprintln!(
            "Не возможно открыть файл {}\n:{}",
            &args.input_file,
            input.unwrap_err()
        );
        return;
    };

    let mut output_file = io::stdout();

    let Ok(input_format) = parse_format(&args.input_format) else {
        eprintln!("Невалидный формат файла 1: {}", &args.input_format);
        return;
    };

    let Ok(output_format) = parse_format(&args.output_format) else {
        eprintln!("Невалидный формат файла 1: {}", &args.output_format);
        return;
    };

    let transactions = parse_tx(&mut input_file, input_format);
    let Ok(txs) = transactions else {
        eprintln!(
            "Ошибка при разборе транзакций файла {}:\n{:?}",
            &args.input_file,
            transactions.unwrap_err()
        );
        return;
    };

    if let Err(err) = dump_tx(&mut output_file, output_format, &txs) {
        eprintln!("Ошибка при конвертации транзакций:\n{}", err);
    }
}
