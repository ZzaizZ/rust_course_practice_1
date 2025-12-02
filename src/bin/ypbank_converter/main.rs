use std::{fmt, io};

use clap::Parser;
use std::fs;
use ypbank_parser::{error, types};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Путь до исходного файла с транзакциями
    #[arg(long, required = true)]
    input_file: String,

    /// Формат исходного файла: text/csv/bin
    #[arg(long, required = true)]
    input_format: KnownFileFormat,

    /// Формат выходного файла: text/csv/bin
    #[arg(long, required = true)]
    output_format: KnownFileFormat,
}

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

fn run() -> Result<(), Error> {
    let args = Args::parse();

    let input_file = fs::File::open(&args.input_file);
    let Ok(mut input_file) = input_file else {
        return Err(Error::Usage(format!(
            "невозможно открыть файл {}\n:{}",
            &args.input_file,
            input_file.unwrap_err()
        )));
    };

    let mut output_file = io::stdout();

    let input_format = args.input_format;
    let output_format = args.output_format;

    let transactions = ypbank_parser::parse(&mut input_file, input_format.as_supported());
    let Ok(transactions) = transactions else {
        return Err(Error::Usage(format!(
            "ошибка при разборе транзакций исходного файла:\n{:?}",
            transactions.unwrap_err()
        )));
    };

    ypbank_parser::dump(
        &mut output_file,
        output_format.as_supported(),
        &transactions,
    )?;

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
