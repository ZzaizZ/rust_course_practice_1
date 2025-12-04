//! Модуль верхнего уровня для парсинга и дампа транзакций.
//!
//! Предоставляет унифицированный интерфейс для работы с различными форматами файлов
//! через функции [`parse`] и [`dump`].

use crate::{error, types};
use std::io;

/// Трейт, который должны реализовывать все парсеры конкретных форматов.
pub(crate) trait Parser {
    /// Читает транзакции из потока.
    fn parse(reader: &mut impl io::Read) -> Result<Vec<types::Transaction>, error::ParseError>;
    /// Записывает транзакции в поток.
    fn dump(
        writer: &mut impl io::Write,
        transactions: &[types::Transaction],
    ) -> Result<(), error::DumpError>;
}

/// Читает список транзакций из предоставленного потока в заданном формате.
///
/// ## Аргументы
///
/// * `reader` - Поток ввода, откуда будут читаться данные (например, файл или буфер).
/// * `format` - Формат данных ([`types::SupportedFileFormat`]), который ожидается в потоке.
///
/// ## Возвращаемое значение
///
/// Возвращает вектор успешно прочитанных транзакций [`types::Transaction`] или ошибку [`error::ParseError`].
///
/// # Пример
///
/// Чтение из строки (используя `as_bytes()`) в CSV формате:
///
/// ```rust
/// use ypbank_parser::{parse, types::{Transaction, SupportedFileFormat}};
///
/// let data = r##"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
///                1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding""##;
/// let mut reader = data.as_bytes();
///
/// let txs = parse(&mut reader, SupportedFileFormat::Csv).expect("Ошибка парсинга");
/// assert_eq!(txs.len(), 1);
/// assert_eq!(txs[0].description, "Initial account funding");
/// ```
///
/// Чтение из файла:
///
/// ```no_run
/// use std::fs::File;
/// use ypbank_parser::{parse, types::{Transaction, SupportedFileFormat}};
///
/// let mut file = File::open("history.txt").expect("Файл не найден");
/// let txs = parse(&mut file, SupportedFileFormat::Text).expect("Ошибка парсинга");
/// ```
pub fn parse(
    reader: &mut impl io::Read,
    format: types::SupportedFileFormat,
) -> Result<Vec<types::Transaction>, error::ParseError> {
    match format {
        types::SupportedFileFormat::Csv => crate::csv_format::CsvParser::parse(reader),
        types::SupportedFileFormat::Bin => crate::bin_format::BinParser::parse(reader),
        types::SupportedFileFormat::Text => crate::text_format::TextParser::parse(reader),
    }
}

/// Записывает список транзакций в предоставленный поток в указанном формате.
///
/// ## Аргументы
///
/// * `writer` - Поток вывода, куда будут записаны данные.
/// * `format` - Целевой формат данных (подробнее о поддерживаемых форматах см. [`types::SupportedFileFormat`]).
/// * `transactions` - Срез транзакций для записи.
///
/// ## Возвращаемое значение
///
/// Возвращает `Ok(())` в случае успеха или ошибку [`error::DumpError`].
///
/// # Пример
///
/// Запись в буфер в памяти в текстовом формате:
///
/// ```rust
/// use ypbank_parser::{dump, types::{Transaction, TxStatus, TxType, SupportedFileFormat}};
///
/// let txs = vec![Transaction{id: 1, r#type: TxType::Deposit,
///                            from_user: 1001, to_user: 1001,
///                            amount: 1001, timestamp: 1633036800000,
///                            status: TxStatus::Success,
///                            description: "Description".to_string()}];
/// let mut buffer = Vec::new();
///
/// dump(&mut buffer, SupportedFileFormat::Text, &txs).expect("Ошибка записи");
///
/// let result_string = String::from_utf8(buffer).expect("Невалидный UTF-8");
/// assert!(result_string.contains("STATUS: SUCCESS"));
/// ```
pub fn dump(
    writer: &mut impl io::Write,
    format: types::SupportedFileFormat,
    transactions: &[types::Transaction],
) -> Result<(), error::DumpError> {
    match format {
        types::SupportedFileFormat::Csv => crate::csv_format::CsvParser::dump(writer, transactions),
        types::SupportedFileFormat::Bin => crate::bin_format::BinParser::dump(writer, transactions),
        types::SupportedFileFormat::Text => {
            crate::text_format::TextParser::dump(writer, transactions)
        }
    }
}
