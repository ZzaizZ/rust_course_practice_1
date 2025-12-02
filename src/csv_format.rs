use std::io::{self, BufRead};

use crate::error;
use crate::types::{Transaction, TxStatus, TxType};

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

fn parse_csv_line(line: &str) -> Result<Vec<String>, error::ParseError> {
    let mut result = Vec::with_capacity(8);
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                result.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }
    if in_quotes {
        return Err(error::ParseError::InvalidFormat(
            "unclosed quotes in CSV line".to_string(),
        ));
    }
    result.push(current.trim().to_string());
    Ok(result)
}

fn parse_header<I: Iterator<Item = io::Result<String>>>(
    lines: &mut I,
) -> Result<Vec<String>, error::ParseError> {
    for line in lines {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        return parse_csv_line(trimmed);
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
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        result.push(parse_transaction(trimmed)?);
    }
    Ok(result)
}

fn parse_transaction(tx: &str) -> Result<Transaction, error::ParseError> {
    let values: Vec<String> = parse_csv_line(tx)?;
    if values.len() != EXPECTED_HEADER.len() {
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
    let description = values[7].clone();

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
        format!("\"{}\"", make_escaped_string(&tx.description)),
    ];
    writeln!(writer, "{}", values.join(","))?;
    Ok(())
}

fn make_escaped_string(input: &str) -> String {
    let mut escaped = String::new();
    for c in input.chars() {
        if c == '"' {
            escaped.push('"');
        }
        escaped.push(c);
    }
    escaped
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let input = r##"
        TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
        1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"String, with ""comma and quotes"""
        1002,WITHDRAWAL,0,501,50000,1672531200000,FAILURE,"simple string"
        "##;

        let expected = &[
            Transaction {
                id: 1001,
                r#type: TxType::Deposit,
                from_user: 0,
                to_user: 501,
                amount: 50000,
                timestamp: 1672531200000,
                status: TxStatus::Success,
                description: r##"String, with "comma and quotes""##.to_string(),
            },
            Transaction {
                id: 1002,
                r#type: TxType::Withdrawal,
                from_user: 0,
                to_user: 501,
                amount: 50000,
                timestamp: 1672531200000,
                status: TxStatus::Failure,
                description: r##"simple string"##.to_string(),
            },
        ];

        let got = parse_from_csv(&mut input.as_bytes());

        assert!(got.is_ok());

        let txs = got.as_ref().unwrap();

        assert_eq!(txs.len(), 2);
        assert_eq!(txs, expected);
    }

    #[test]
    fn test_parse_mailformed() {
        let input = r##"
        TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
        1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS
        "##;

        let got = parse_from_csv(&mut input.as_bytes());

        assert!(got.is_err());
    }

    #[test]
    fn test_parse_string() {
        let input = r##"
        TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
        1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"String with ""quotes"""
        "##;

        let expected_description = r##"String with "quotes""##.to_string();

        let got = parse_from_csv(&mut input.as_bytes());

        assert!(got.is_ok());

        let txs = got.as_ref().unwrap();

        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].description, expected_description);
    }

    #[test]
    fn test_parse_mailformed_string() {
        let input = r##"
        TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
        1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"String with unclosed quotes
        "##;

        let got = parse_from_csv(&mut input.as_bytes());

        assert!(got.is_err());
    }

    #[test]
    fn test_escaped_string() {
        let input = r##"String with "quotes" and , commas"##;
        let expected = r##"String with ""quotes"" and , commas"##.to_string();
        let got = make_escaped_string(input);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_dump_transaction() {
        let txs = vec![
            Transaction {
                id: 1001,
                r#type: TxType::Deposit,
                from_user: 1001,
                to_user: 1001,
                amount: 1001,
                timestamp: 1633036800000,
                status: TxStatus::Success,
                description: "Description".to_string(),
            },
            Transaction {
                id: 1002,
                r#type: TxType::Deposit,
                from_user: 1001,
                to_user: 1001,
                amount: 1001,
                timestamp: 1633036800000,
                status: TxStatus::Success,
                description: r##"Description with, comma and "quotes""##.to_string(),
            },
        ];
        let mut buffer = Vec::new();

        let dump_result = dump_as_csv(&mut buffer, &txs);
        assert!(dump_result.is_ok());

        let result_string = String::from_utf8(buffer).expect("Невалидный UTF-8");
        assert!(
            result_string
                .contains(r##"1001,DEPOSIT,1001,1001,1001,1633036800000,SUCCESS,"Description""##)
        );

        let lines: Vec<&str> = result_string.lines().collect();

        assert_eq!(
            lines[0],
            "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION",
        );
        assert_eq!(
            lines[1],
            "1001,DEPOSIT,1001,1001,1001,1633036800000,SUCCESS,\"Description\"",
        );
        assert_eq!(
            lines[2],
            "1002,DEPOSIT,1001,1001,1001,1633036800000,SUCCESS,\"Description with, comma and \"\"quotes\"\"\"",
        );
    }
}
