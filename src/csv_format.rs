use std::io::{self, BufRead};

use crate::error;
use crate::types::{Transaction, TxStatus, TxType};
use crate::utils::wrap_with_quotes;

const EXPECTED_HEADER: &[&str] = &[
    "TX_ID",
    "TX_TYPE",
    "FROM_USER_ID",
    "TO_USER_ID",
    "AMOUNT",
    "TIMESTAMP",
    "STATUS",
    "DESCRIPTION",
];

/// Читает и парсит транзакции из формата CSV.
///
/// # Аргументы
///
/// * `reader` - Источник данных. Это может быть открытый файл, сетевой поток или
///   массив байт. Должен реализовывать трейт [`std::io::Read`].  
///   Данные должны быть в текстовом формате ([doc/YPBankTextFormat_ru.md](doc/YPBankCsvFormat_ru.md))
///
/// # Ошибки
///
/// Возвращает [`ParseError`], если:
/// * Формат данных некорректен.
/// * Возникла ошибка ввода-вывода при чтении из `reader`.
///
/// # Пример
///
/// Чтение из строки (используя `as_bytes()`):
///
/// ```rust
/// use ypbank_parser::{parse_from_csv, types::Transaction};
///
/// let data = r##"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
///                1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial account funding""##;
/// let mut reader = data.as_bytes();
///
/// let txs = parse_from_csv(&mut reader).expect("Ошибка парсинга");
/// assert_eq!(txs.len(), 1);
/// assert_eq!(txs[0].description, "Initial account funding");
/// ```
pub fn parse_from_csv(reader: &mut impl io::Read) -> Result<Vec<Transaction>, error::ParseError> {
    let mut lines = io::BufReader::new(reader).lines();
    let header_types = parse_header(&mut lines)?;
    if !header_is_valid(&header_types) {
        return Err(error::ParseError::InvalidFormat(
            "invalid header".to_string(),
        ));
    }
    parse_transactions(&mut lines)
}

fn parse_header<I: Iterator<Item = io::Result<String>>>(
    lines: &mut I,
) -> Result<Vec<String>, error::ParseError> {
    for line in lines {
        let l = line?.trim().to_string();
        if l.is_empty() {
            continue;
        }
        return Ok(l.split_terminator(',').map(|s| s.to_string()).collect());
    }
    Err(error::ParseError::InvalidFormat(
        "invalid header".to_string(),
    ))
}

fn header_is_valid(header: &Vec<String>) -> bool {
    EXPECTED_HEADER == header
}

fn parse_transactions<I: Iterator<Item = io::Result<String>>>(
    lines: &mut I,
) -> Result<Vec<Transaction>, error::ParseError> {
    let mut result = Vec::<Transaction>::new();
    for line in lines {
        let l = line?.trim().to_string();
        if l.is_empty() {
            continue;
        }
        result.push(parse_transaction(&l)?);
    }
    Ok(result)
}

fn parse_transaction(tx: &str) -> Result<Transaction, error::ParseError> {
    let values: Vec<&str> = tx.split(',').collect();
    if values.len() < EXPECTED_HEADER.len() {
        return Err(error::ParseError::InvalidFormat(format!(
            "invalid fields count: {}",
            values.len()
        )));
    }

    let id = values[0].parse::<u64>()?;
    let r#type = values[1].parse::<TxType>()?;
    let from_user = values[2].parse::<u64>()?;
    let to_user = values[3].parse::<u64>()?;
    let amount = values[4].parse::<u64>()?;
    let timestamp = values[5].parse::<u64>()?;
    let status = values[6].parse::<TxStatus>()?;
    let description = crate::utils::strip_quotes(values[7..].join(","));

    Ok(Transaction {
        id,
        r#type,
        from_user,
        to_user,
        amount,
        timestamp,
        status,
        description,
    })
}

/// Сериализует список транзакций в формат CSV, записывая результат в `writer`.
///
/// # Аргументы
///
/// * `writer` - Приемник данных. Это может быть файл, сокет или буфер в памяти (`Vec<u8>`).
///   Должен реализовывать трейт [`std::io::Write`].
/// * `transactions` - Слайс транзакций для записи.
///
/// # Ошибки
///
/// Возвращает [`DumpError`], если:
/// * Произошла ошибка ввода-вывода (IO error) при записи во `writer`.
///
/// # Пример
///
/// Запись в буфер в памяти:
///
/// ```rust
/// use ypbank_parser::{dump_as_csv, types::{Transaction, TxStatus, TxType}, };
///
/// let txs = vec![Transaction{id: 1, r#type: TxType::Deposit,
///                            from_user: 1001, to_user: 1001,
///                            amount: 1001, timestamp: 1633036800000,
///                            status: TxStatus::Success,
///                            description: "Description".to_string()}];
/// let mut buffer = Vec::new();
///
/// dump_as_csv(&mut buffer, &txs).expect("Ошибка записи");
///
/// let result_string = String::from_utf8(buffer).expect("Невалидный UTF-8");
/// assert!(result_string.contains("1,DEPOSIT,1001,1001,1001,1633036800000,SUCCESS,\"Description\""));
/// ```
pub fn dump_as_csv(
    writer: &mut impl io::Write,
    transactions: &[Transaction],
) -> Result<(), error::DumpError> {
    write_title(writer)?;
    for tx in transactions {
        write_tx(writer, tx)?;
    }
    Ok(())
}

fn write_title(writer: &mut impl io::Write) -> Result<(), error::DumpError> {
    let title = EXPECTED_HEADER.join(",");
    writeln!(writer, "{}", title)?;
    Ok(())
}

fn write_tx(writer: &mut impl io::Write, tx: &Transaction) -> Result<(), error::DumpError> {
    let values = [
        tx.id.to_string(),
        tx.r#type.to_string(),
        tx.from_user.to_string(),
        tx.to_user.to_string(),
        tx.amount.to_string(),
        tx.timestamp.to_string(),
        tx.status.to_string(),
        wrap_with_quotes(&tx.description),
    ];
    writeln!(writer, "{}", values.join(","))?;
    Ok(())
}
