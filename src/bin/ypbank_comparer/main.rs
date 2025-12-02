use clap::Parser;
use core::fmt;
use std::{fs, io};
use ypbank_parser::{
    error, parse_from_bin, parse_from_csv, parse_from_text,
    types::{self, Transaction},
};

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

impl Error {
    fn code(&self) -> i32 {
        match self {
            Self::Parse(_) => 1,
            Self::Dump(_) => 2,
            Self::Usage(_) => 3,
            Self::IO => 4,
        }
    }
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
        _ => Err(Error::Usage("unknown format".to_string())),
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

#[derive(Parser, Debug)]
struct Args {
    /// Input file path
    #[arg(long, required = true)]
    file1: String,

    /// Input file type: text/csv/bin
    #[arg(long, required = true)]
    format1: String,

    /// Input file path
    #[arg(long, required = true)]
    file2: String,

    /// Output file type: text/csv/bin
    #[arg(long, required = true)]
    format2: String,
}

// Сравнивает набор транзакций.
// Возвращает либо:
// - None, если наборы идентичны
// - (index, Option<&'a Transaction>, Option<&'a Transaction>), первой несовпавшей пары транзакций
fn compare<'a>(
    lhs: &'a [Transaction],
    rhs: &'a [Transaction],
) -> Option<(usize, Option<&'a Transaction>, Option<&'a Transaction>)> {
    for i in 0..std::cmp::max(lhs.len(), rhs.len()) {
        let l = lhs.get(i);
        let r = rhs.get(i);
        if l.is_none() || r.is_none() || l.unwrap() != r.unwrap() {
            return Some((i, l, r));
        }
    }
    None
}

fn run() -> Result<(), Error> {
    let args = Args::parse();

    let file1 = fs::File::open(&args.file1);
    let Ok(mut f1) = file1 else {
        return Err(Error::Usage(format!(
            "невозможно открыть файл {}\n:{}",
            &args.file1,
            file1.unwrap_err()
        )));
    };

    let file2 = fs::File::open(&args.file2);
    let Ok(mut f2) = file2 else {
        return Err(Error::Usage(format!(
            "невозможно открыть файл {}\n:{}",
            &args.file2,
            file2.unwrap_err()
        )));
    };

    let Ok(format1) = parse_format(&args.format1) else {
        return Err(Error::Usage(format!(
            "невалидный формат файла 1: {}",
            &args.format1
        )));
    };

    let Ok(format2) = parse_format(&args.format2) else {
        return Err(Error::Usage(format!(
            "невалидный формат файла 2: {}",
            &args.format2
        )));
    };

    let transactions1 = parse_tx(&mut f1, format1);
    let Ok(tx1_unwraped) = transactions1 else {
        return Err(Error::Usage(format!(
            "ошибка при разборе транзакций файла 1:\n{:?}",
            transactions1.unwrap_err()
        )));
    };
    let transactions2 = parse_tx(&mut f2, format2);
    let Ok(tx2_unwraped) = transactions2 else {
        return Err(Error::Usage(format!(
            "ошибка при разборе транзакций файла 2:\n{:?}",
            transactions2.unwrap_err()
        )));
    };

    let result = compare(&tx1_unwraped, &tx2_unwraped);
    if let Some(r) = &result {
        println!("Наборы транзакций не иднетичны!");
        println!("Несовпали транзакции на позииции {}", r.0 + 1);

        println!("LHS:\n{:#?}\n\nRHS:\n{:#?}", r.1, r.2);
    } else {
        println!("Наборы транзакций идентичны!")
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(e.code());
        }
    }
}
