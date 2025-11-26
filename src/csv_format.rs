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

pub fn parse_from_csv(reader: &mut impl io::Read) -> Result<Vec<Transaction>, error::ParseError> {
    let mut lines = io::BufReader::new(reader).lines();
    let header_types = parse_header(&mut lines)?;
    if !header_is_valid(&header_types) {
        return Err(error::ParseError::InvalidFormat);
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
    Err(error::ParseError::InvalidFormat)
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
    if values.len() != EXPECTED_HEADER.len() {
        return Err(error::ParseError::InvalidFormat);
    }

    let id = values[0].parse::<u64>()?;
    let r#type = values[1].parse::<TxType>()?;
    let from_user = values[2].parse::<u64>()?;
    let to_user = values[3].parse::<u64>()?;
    let amount = values[4].parse::<u64>()?;
    let timestamp = values[5].parse::<u64>()?;
    let status = values[6].parse::<TxStatus>()?;
    let description = values[7].to_string();

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
        tx.description.clone(),
    ];
    writeln!(writer, "{}", values.join(","))?;
    Ok(())
}
