use crate::error::{self, DumpError, ParseError};
use crate::types::{Transaction, TxStatus, TxType};
use crate::utils;
use core::fmt;
use std::collections::HashMap;
use std::{
    io::{self, BufRead},
    str::FromStr,
};

trait Validator {
    fn is_valid(&self) -> bool;
}

static REQUIRED_FIELDS: &[&str] = &[
    "TX_ID",
    "TX_TYPE",
    "FROM_USER_ID",
    "TO_USER_ID",
    "AMOUNT",
    "TIMESTAMP",
    "STATUS",
    "DESCRIPTION",
];

struct TxWrapper {
    parsed_fields: HashMap<String, String>,
}

impl TxWrapper {
    fn new() -> Self {
        Self {
            parsed_fields: HashMap::with_capacity(8),
        }
    }

    fn from_tx(tx: &Transaction) -> Self {
        let mut fields = HashMap::<String, String>::with_capacity(8);
        fields.insert("TX_ID".to_string(), tx.id.to_string());
        fields.insert("TX_TYPE".to_string(), tx.r#type.to_string());
        fields.insert("FROM_USER_ID".to_string(), tx.from_user.to_string());
        fields.insert("TO_USER_ID".to_string(), tx.to_user.to_string());
        fields.insert("AMOUNT".to_string(), tx.amount.to_string());
        fields.insert("TIMESTAMP".to_string(), tx.timestamp.to_string());
        fields.insert("STATUS".to_string(), tx.status.to_string());
        fields.insert("DESCRIPTION".to_string(), tx.description.clone());

        TxWrapper {
            parsed_fields: fields,
        }
    }

    fn apply_field(&mut self, name: &str, value: &str) -> Result<(), ParseError> {
        if self.parsed_fields.contains_key(name) {
            return Err(ParseError::InvalidFormat(format!(
                "duplicate field {}",
                name
            )));
        }
        self.parsed_fields
            .insert(name.to_string(), value.to_string());
        Ok(())
    }

    fn build(&self) -> Result<Transaction, ParseError> {
        let id: u64 = self.parsed_fields["TX_ID"].parse()?;
        let r#type: TxType = self.parsed_fields["TX_TYPE"].parse()?;
        let from_user: u64 = self.parsed_fields["FROM_USER_ID"].parse()?;
        let to_user: u64 = self.parsed_fields["TO_USER_ID"].parse()?;
        let amount: u64 = self.parsed_fields["AMOUNT"].parse()?;
        let timestamp: u64 = self.parsed_fields["TIMESTAMP"].parse()?;
        let status: TxStatus = self.parsed_fields["STATUS"].parse()?;
        let description = utils::parse_quoted_field(&self.parsed_fields["DESCRIPTION"]);

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
}

fn dump_txw_as_text(txw: &TxWrapper, writer: &mut impl io::Write) -> Result<(), error::DumpError> {
    REQUIRED_FIELDS.iter().try_for_each(|s| {
        let Some(val) = txw.parsed_fields.get(*s) else {
            return Err(DumpError::InternalError);
        };
        if *s == "DESCRIPTION" {
            writeln!(writer, "{}: {}", s, utils::wrap_with_quotes(val))?;
            Ok(())
        } else {
            writeln!(writer, "{}: {}", s, val)?;
            Ok(())
        }
    })?;
    Ok(())
}

impl Validator for TxWrapper {
    fn is_valid(&self) -> bool {
        REQUIRED_FIELDS
            .iter()
            .all(|required_field| self.parsed_fields.contains_key(*required_field))
    }
}

impl FromStr for TxType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DEPOSIT" => Ok(TxType::Deposit),
            "TRANSFER" => Ok(TxType::Transfer),
            "WITHDRAWAL" => Ok(TxType::Withdrawal),
            _ => Err(ParseError::InvalidFormat("unknown tx type".to_string())),
        }
    }
}

impl FromStr for TxStatus {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SUCCESS" => Ok(TxStatus::Success),
            "FAILURE" => Ok(TxStatus::Failure),
            "PENDING" => Ok(TxStatus::Pending),
            _ => Err(ParseError::InvalidFormat("unknown tx status".to_string())),
        }
    }
}

fn parse_lines<I: Iterator<Item = io::Result<String>>>(
    lines: I,
) -> Result<Vec<Transaction>, ParseError> {
    let mut result: Vec<Transaction> = Vec::new();
    let mut current_tx = TxWrapper::new();
    for line in lines {
        let l = line?.trim().to_string();
        if l.is_empty() {
            if !current_tx.is_valid() {
                current_tx = TxWrapper::new();
                continue;
            }
            result.push(current_tx.build()?);
            continue;
        }
        let parts: Vec<&str> = l.split(':').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(ParseError::InvalidFormat(
                "invalid field format".to_string(),
            ));
        }
        current_tx.apply_field(parts[0], parts[1])?;
    }

    if current_tx.is_valid() {
        result.push(current_tx.build()?);
    }
    Ok(result)
}

/// Читает и парсит транзакции из текстового формата.
///
/// # Аргументы
///
/// * `reader` - Источник данных. Это может быть открытый файл, сетевой поток или
///   массив байт. Должен реализовывать трейт [`std::io::Read`].  
///   Данные должны быть в текстовом формате ([doc/YPBankTextFormat_ru.md](doc/YPBankTextFormat_ru.md))
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
/// use ypbank_parser::{parse_from_text, types::Transaction};
///
/// let data = r##"TX_ID: 123
///                TX_TYPE: DEPOSIT
///                FROM_USER_ID: 0
///                TO_USER_ID: 9876543210987654
///                AMOUNT: 10000
///                TIMESTAMP: 1633036800000
///                STATUS: SUCCESS
///                DESCRIPTION: "Terminal deposit""##;
/// let mut reader = data.as_bytes();
///
/// let txs = parse_from_text(&mut reader).expect("Ошибка парсинга");
/// assert_eq!(txs.len(), 1);
/// ```
///
/// Чтение из файла:
///
/// ```no_run
/// use std::fs::File;
/// use ypbank_parser::parse_from_text;
///
/// let mut file = File::open("history.txt").expect("Файл не найден");
/// let txs = parse_from_text(&mut file).expect("Ошибка парсинга");
/// ```
pub fn parse_from_text(reader: &mut impl io::Read) -> Result<Vec<Transaction>, ParseError> {
    let lines = io::BufReader::new(reader).lines();
    parse_lines(lines)
}

impl fmt::Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deposit => write!(f, "DEPOSIT"),
            Self::Transfer => write!(f, "TRANSFER"),
            Self::Withdrawal => write!(f, "WITHDRAWAL"),
        }
    }
}

impl fmt::Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "SUCCESS"),
            Self::Failure => write!(f, "FAILURE"),
            Self::Pending => write!(f, "PENDING"),
        }
    }
}

/// Сериализует список транзакций в текстовый формат, записывая результат в `writer`.
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
/// use ypbank_parser::{dump_as_text, types::{Transaction, TxStatus, TxType}, };
///
/// let txs = vec![Transaction{id: 1, r#type: TxType::Deposit,
///                            from_user: 1001, to_user: 1001,
///                            amount: 1001, timestamp: 1633036800000,
///                            status: TxStatus::Success,
///                            description: "Description".to_string()}];
/// let mut buffer = Vec::new();
///
/// dump_as_text(&mut buffer, &txs).expect("Ошибка записи");
///
/// let result_string = String::from_utf8(buffer).expect("Невалидный UTF-8");
/// assert!(result_string.contains("STATUS: SUCCESS"));
/// ```
pub fn dump_as_text(
    writer: &mut impl io::Write,
    transactions: &[Transaction],
) -> Result<(), DumpError> {
    let mut iter = transactions.iter().peekable();
    while let Some(tx) = iter.next() {
        let txw = TxWrapper::from_tx(tx);
        dump_txw_as_text(&txw, writer)?;
        if iter.peek().is_some() {
            writeln!(writer)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_one_valid_transaction() {
        let input = r##"TX_ID: 123
                           TX_TYPE: DEPOSIT
                           FROM_USER_ID: 0
                           TO_USER_ID: 9876543210987654
                           AMOUNT: 10000
                           TIMESTAMP: 1633036800000
                           STATUS: SUCCESS
                           DESCRIPTION: "Terminal deposit""##;

        let expected = Transaction {
            id: 123,
            r#type: crate::types::TxType::Deposit,
            from_user: 0,
            to_user: 9876543210987654,
            amount: 10000,
            timestamp: 1633036800000,
            status: TxStatus::Success,
            description: "Terminal deposit".to_string(),
        };

        let got = parse_from_text(&mut input.as_bytes());

        assert!(got.as_ref().err().is_none());

        let txs = got.as_ref().unwrap();
        assert_eq!(txs.len(), 1);

        assert_eq!(expected, txs[0]);
    }

    #[test]
    fn test_dump_validtransaction() {
        let input: Vec<Transaction> = vec![Transaction {
            id: 123,
            r#type: TxType::Deposit,
            from_user: 0,
            to_user: 9876543210987654,
            amount: 10000,
            timestamp: 1633036800000,
            status: TxStatus::Success,
            description: "Terminal deposit".to_string(),
        }];

        let mut got = Vec::new();

        let res = dump_as_text(&mut got, &input);

        assert!(res.as_ref().err().is_none());

        validate(&String::from_utf8_lossy(&got));
    }

    fn validate(got: &str) {
        let expected = vec![
            "TX_ID: 123",
            "TX_TYPE: DEPOSIT",
            "FROM_USER_ID: 0",
            "TO_USER_ID: 9876543210987654",
            "AMOUNT: 10000",
            "TIMESTAMP: 1633036800000",
            "STATUS: SUCCESS",
            "DESCRIPTION: \"Terminal deposit\"",
        ];

        for ex in expected {
            assert!(got.contains(ex));
        }
    }

    #[test]
    fn test_duplicate_field() {
        let input = r##"TX_ID: 123
                           TX_TYPE: DEPOSIT
                           FROM_USER_ID: 0
                           TO_USER_ID: 9876543210987654
                           AMOUNT: 10000
                           TIMESTAMP: 1633036800000
                           STATUS: SUCCESS
                           DESCRIPTION: "Terminal deposit"
                           DESCRIPTION: "Duplicate field""##;

        let got = parse_from_text(&mut input.as_bytes());

        assert!(got.is_err());
    }
}
