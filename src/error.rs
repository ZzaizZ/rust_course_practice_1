//! Модуль обработки ошибок.
//!
//! Содержит типы ошибок, используемые при операциях чтения ([`ParseError`]) и записи ([`DumpError`])
//! транзакций. Эти ошибки унифицируют сбои, возникающие в различных форматах (CSV, BIN, Text).

use std::num::ParseIntError;

/// Ошибки, возникающие при парсинге (десериализации) данных.
///
/// Используется функциями `parse_from_*` в модулях [`crate::csv_format`],
/// [`crate::bin_format`] и [`crate::text_format`].
#[derive(Debug)]
pub enum ParseError {
    /// Ошибка ввода-вывода, возникшая при чтении из источника (например, `std::io::Error`).
    /// Содержит строковое описание ошибки.
    IOError(String),
    /// Ошибка валидации формата данных.
    ///
    /// Может возникать в случаях:
    /// - Неверный заголовок CSV или количество полей.
    /// - Формат значения (например, ожидался u64, но получена строка).
    /// - Несовпадение сигнатуры в начале записи в BIN формате.
    /// - Дублирующиеся поля или неизвестные значения перечислений в Text формате.
    InvalidFormat(String),
}

impl From<std::io::Error> for ParseError {
    fn from(value: std::io::Error) -> Self {
        ParseError::IOError(value.to_string())
    }
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        ParseError::InvalidFormat(err.to_string())
    }
}

/// Ошибки, возникающие при дампе (сериализации) данных.
///
/// Используется функциями `dump_as_*` для записи транзакций в поток.
#[derive(Debug)]
pub enum DumpError {
    /// Внутренняя ошибка логики сериализации.
    /// Возникает, если структура данных находится в несогласованном состоянии
    /// (например, отсутствуют обязательные поля при формировании текстового вывода).
    InternalError,
    /// Ошибка ввода-вывода при записи в целевой поток (например, ошибка записи в файл).
    OutputError,
}

impl From<std::io::Error> for DumpError {
    fn from(_: std::io::Error) -> Self {
        DumpError::OutputError
    }
}
