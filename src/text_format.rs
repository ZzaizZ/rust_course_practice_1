use crate::error::{DumpError, ParseError};
use crate::types::{Transaction, TxStatus, TxType};
use core::fmt;
use std::{
    collections::HashSet,
    io::{self, BufRead, Write},
    str::FromStr,
};

trait Validator {
    fn is_valid(&self) -> bool;
}

static REQUIRED_FIELDS: &'static [&'static str] = &[
    "TX_ID",
    "TX_TYPE",
    "FROM_USER_ID",
    "TO_USER_ID",
    "AMOUNT",
    "TIMESTAMP",
    "STATUS",
    "DESCRIPTION",
];

struct TransactionWrapper {
    tx: Transaction,
    parsed_fields: HashSet<String>,
}

impl TransactionWrapper {
    fn new() -> Self {
        let mut fields = HashSet::new();
        fields.reserve(REQUIRED_FIELDS.len());
        TransactionWrapper {
            tx: Transaction::default(),
            parsed_fields: fields,
        }
    }

    fn release(&mut self) -> Transaction {
        std::mem::take(&mut self.tx)
    }

    fn apply_field(&mut self, name: &str, value: &str) -> Result<(), ParseError> {
        match name {
            "TX_ID" => self.tx.id = value.parse()?,
            "FROM_USER_ID" => self.tx.from_user = value.parse()?,
            "TX_TYPE" => self.tx.r#type = value.parse()?,
            "TO_USER_ID" => self.tx.to_user = value.parse()?,
            "AMOUNT" => self.tx.amount = value.parse()?,
            "TIMESTAMP" => self.tx.timestamp = value.parse()?,
            "STATUS" => self.tx.status = value.parse()?,
            "DESCRIPTION" => self.tx.description = value.trim_matches('"').to_string(),
            _ => return Err(ParseError::InvalidFormat),
        }
        
        self.parsed_fields.insert(name.to_string());
        Ok(())
    }
}

impl Validator for TransactionWrapper {
    fn is_valid(&self) -> bool {
        REQUIRED_FIELDS
            .iter()
            .all(|required_field| self.parsed_fields.contains(*required_field))
    }
}

impl FromStr for TxType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DEPOSIT" => Ok(TxType::Deposit),
            "TRANSFER" => Ok(TxType::Transfer),
            "WITHDRAWAL" => Ok(TxType::Withdrawal),
            _ => Err(ParseError::InvalidFormat),
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
            _ => Err(ParseError::InvalidFormat),
        }
    }
}

fn parse_lines<I: Iterator<Item = io::Result<String>>>(
    lines: I,
) -> Result<Vec<Transaction>, ParseError> {
    let mut result: Vec<Transaction> = Vec::new();
    let mut current_tx: TransactionWrapper = TransactionWrapper::new();
    for line in lines {
        let l = line?.trim().to_string();
        if l.is_empty() {
            if !current_tx.is_valid() {
                current_tx = TransactionWrapper::new();
                continue;
            }
            result.push(current_tx.release());
            continue;
        }
        let parts: Vec<&str> = l.split(':').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(ParseError::InvalidFormat);
        }
        current_tx.apply_field(parts[0], parts[1])?;
    }

    if current_tx.is_valid() {
        result.push(current_tx.release());
    }
    Ok(result)
}

pub fn parse_from_text<R: io::Read>(reader: R) -> Result<Vec<Transaction>, ParseError> {
    let lines = io::BufReader::new(reader).lines();
    parse_lines(lines)
}

impl fmt::Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deposit => write!(f, "DEPOSIT"),
            Self::Transfer => write!(f, "TRANSFER"),
            Self::Withdrawal => write!(f, "WITHDRAWAL"),
            Self::Unknown => write!(f, "UNKNOWN"), //FXIME: надо дропнуть
        }
    }
}

impl fmt::Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "SUCCESS"),
            Self::Failure => write!(f, "FAILURE"),
            Self::Pending => write!(f, "PENDING"),
            Self::Unknown => write!(f, "UNKNOWN"), //FIXME: надо дропнуть
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TX_ID: {}\nTX_TYPE: {}\nFROM_USER_ID: {}\nTO_USER_ID: {}\nAMOUNT: {}\nTIMESTAMP: {}\nSTATUS: {}\nDESCRIPTION: \"{}\"",
            self.id,
            self.r#type,
            self.from_user,
            self.to_user,
            self.amount,
            self.timestamp,
            self.status,
            self.description
        )
    }
}

pub fn dump_as_text<W: io::Write>(
    writer: &mut W,
    transactions: &Vec<Transaction>,
) -> Result<(), DumpError> {
    let mut w = io::BufWriter::new(writer);
    for tx in transactions {
        if let Err(_) = write!(w, "{}\n", tx) {
            return Err(DumpError::InternalError);
        }
    }
    Ok(())
}

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
            r#type: TxType::Deposit,
            from_user: 0,
            to_user: 9876543210987654,
            amount: 10000,
            timestamp: 1633036800000,
            status: TxStatus::Success,
            description: "Terminal deposit".to_string(),
        };

        let got = parse_from_text(input.as_bytes());

        assert!(matches!(got.as_ref().err(), None));

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

        assert!(matches!(res.as_ref().err(), None));

        let expected = "TX_ID: 123\nTX_TYPE: DEPOSIT\nFROM_USER_ID: 0\nTO_USER_ID: 9876543210987654\nAMOUNT: 10000\nTIMESTAMP: 1633036800000\nSTATUS: SUCCESS\nDESCRIPTION: \"Terminal deposit\"\n";

        assert_eq!(expected, String::from_utf8_lossy(&got));

    }
}
