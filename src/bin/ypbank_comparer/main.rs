use clap::Parser;
use core::fmt;
use std::{fs, io};
use ypbank_parser::{
    error,
    types::{self, Transaction},
};

#[derive(clap::ValueEnum, Clone, Debug)]
enum KnownFileFormat {
    Bin,
    Csv,
    Text,
}

impl KnownFileFormat {
    fn as_supported(&self) -> types::SupportedFileFormat {
        match self {
            KnownFileFormat::Bin => types::SupportedFileFormat::Bin,
            KnownFileFormat::Csv => types::SupportedFileFormat::Csv,
            KnownFileFormat::Text => types::SupportedFileFormat::Text,
        }
    }
}

#[derive(Debug)]
enum Error {
    Parse(String),
    Dump(String),
    Usage(String),
    IO(String),
}

impl Error {
    fn code(&self) -> i32 {
        match self {
            Self::Parse(_) => 1,
            Self::Dump(_) => 2,
            Self::Usage(_) => 3,
            Self::IO(_) => 4,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(msg) | Self::Dump(msg) | Self::Usage(msg) => {
                write!(f, "{}", msg)
            }
            Self::IO(msg) => write!(f, "IO error: {}", msg),
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
    fn from(err: io::Error) -> Self {
        Error::IO(format!("ошибка ввода-вывода: {}", err))
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// Input file path
    #[arg(long, required = true)]
    file1: String,

    /// Input file type: text/csv/bin
    #[arg(long, required = true)]
    format1: KnownFileFormat,

    /// Input file path
    #[arg(long, required = true)]
    file2: String,

    /// Output file type: text/csv/bin
    #[arg(long, required = true)]
    format2: KnownFileFormat,
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

    let transactions1 = ypbank_parser::parse(&mut f1, args.format1.as_supported());
    let Ok(tx1_unwraped) = transactions1 else {
        return Err(Error::Usage(format!(
            "ошибка при разборе транзакций файла 1:\n{:?}",
            transactions1.unwrap_err()
        )));
    };
    let transactions2 = ypbank_parser::parse(&mut f2, args.format2.as_supported());
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
