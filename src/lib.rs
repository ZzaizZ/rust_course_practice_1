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
//! use ypbank_parser::{parse_from_csv, dump_as_bin, types::Transaction};
//!
//! let data = r##"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
//!                1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding""##;
//! let mut reader = data.as_bytes();
//!
//! let mut writer = Vec::new();
//!
//! // Парсинг CSV формата
//! let txs = parse_from_csv(&mut reader).expect("Ошибка парсинга");
//! dump_as_bin(&mut writer, &txs).expect("Ошибка записи");
//! ```
//!
//! ## Обработка ошибок
//! Все функции парсинга и дампа возвращают [`Result`], который содержит либо успешный результат,
//! либо ошибки одного из типов [`error::ParseError`, `error::DumpError`].

pub mod error;
pub mod types;

mod bin_format;
mod csv_format;
mod text_format;
mod utils;

pub use text_format::{dump_as_text, parse_from_text};

pub use bin_format::{dump_as_bin, parse_from_bin};

pub use csv_format::{dump_as_csv, parse_from_csv};
