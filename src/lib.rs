#![warn(missing_docs)]
//! # ypbank_parser
//!
//! Библиотека для парсинга и дампа истории транзакций в различных форматах.
//!
//! Этот крейт предоставляет унифицированный интерфейс для работы с тремя основными форматами:
//! * **CSV** (описание формата в [doc/YPBankCsvFormat_ru.md](doc/YPBankCsvFormat_ru.md))
//! * **BIN** (описание формата в [doc/YPBankBinFormat_ru.md](doc/YPBankBinFormat_ru.md))
//! * **Text** (описание формата в [doc/YPBankTextFormat_ru.md](doc/YPBankTextFormat_ru.md))
//!
//! ## Быстрый старт
//!
//! ```rust
//! use ypbank_parser::{parse, dump, types::{Transaction, SupportedFileFormat}};
//!
//! let data = r##"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
//!                1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding""##;
//! let mut reader = data.as_bytes();
//!
//! let mut writer = Vec::new();
//!
//! // Парсинг CSV формата
//! let txs = parse(&mut reader, SupportedFileFormat::Csv).expect("Ошибка парсинга");
//! // Сохранение в бинарном формате
//! dump(&mut writer, SupportedFileFormat::Bin, &txs).expect("Ошибка записи");
//! ```
//!
//! ## Обработка ошибок
//! Функции парсинга и дампа возвращают [`Result`], который содержит либо успешный результат,
//! либо ошибки одного из типов [`error::ParseError`, `error::DumpError`] в зависимости от типа операции.

pub mod error;
pub mod types;

mod bin_format;
mod csv_format;
mod parser;
mod text_format;
mod utils;

pub use parser::{dump, parse};
