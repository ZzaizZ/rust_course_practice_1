use crate::error;
use std::{io, mem};

use crate::types::{Transaction, TxStatus, TxType};

const MAGIC: [u8; 4] = [0x59, 0x50, 0x42, 0x4E];

fn read_magic(reader: &mut impl io::Read) -> io::Result<[u8; 4]> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

fn read_u32(reader: &mut impl io::Read) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_u64(reader: &mut impl io::Read) -> io::Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

fn read_string(size: usize, reader: &mut impl io::Read) -> io::Result<String> {
    let mut buf = vec![0u8; size];
    reader.read_exact(&mut buf)?;
    let s = String::from_utf8(buf)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

    Ok(s)
}

fn read_tx_type(reader: &mut impl io::Read) -> io::Result<TxType> {
    let mut buf = vec![0u8; 1];
    reader.read_exact(&mut buf)?;
    match buf[0] {
        0 => Ok(TxType::Deposit),
        1 => Ok(TxType::Transfer),
        2 => Ok(TxType::Withdrawal),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid TxType")),
    }
}

fn read_tx_status(reader: &mut impl io::Read) -> io::Result<TxStatus> {
    let mut buf = vec![0u8; 1];
    reader.read_exact(&mut buf)?;
    match buf[0] {
        0 => Ok(TxStatus::Success),
        1 => Ok(TxStatus::Failure),
        2 => Ok(TxStatus::Pending),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected TxType",
        )),
    }
}

struct Header {
    _magic: [u8; 4],
    record_size: u32,
}

impl Header {
    fn read(reader: &mut impl io::Read) -> io::Result<Self> {
        let magic = read_magic(reader)?;
        if magic != MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid magic"));
        }
        let record_size = read_u32(reader)?;
        Ok(Header {
            _magic: magic,
            record_size,
        })
    }

    fn new(size: u32) -> Self {
        Header {
            _magic: MAGIC,
            record_size: size,
        }
    }

    fn dump(&self) -> Vec<u8> {
        let mut res = Vec::<u8>::with_capacity(Header::sizeof());
        res.extend_from_slice(&self._magic);
        res.extend_from_slice(&self.record_size.to_be_bytes());
        res
    }

    const fn sizeof() -> usize {
        4 + mem::size_of::<u32>()
    }
}

fn read_tx(reader: &mut impl io::Read) -> Result<Transaction, error::ParseError> {
    let id = read_u64(reader)?;
    let r#type = read_tx_type(reader)?;
    let from_user = read_u64(reader)?;
    let to_user = read_u64(reader)?;
    let amount = read_u64(reader)?;
    let timestamp = read_u64(reader)?;
    let status = read_tx_status(reader)?;
    let desc_len = read_u32(reader)?;
    let description = read_string(desc_len as usize, reader)?;

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

pub fn parse_from_bin<R: io::Read>(reader: &mut R) -> Result<Vec<Transaction>, error::ParseError> {
    let mut result = Vec::<Transaction>::new();
    loop {
        match Header::read(reader) {
            Ok(header) => {
                let mut buf = vec![0u8; header.record_size as usize];
                reader.read_exact(&mut buf)?;
                let tx = read_tx(reader)?;
                result.push(tx);
            }
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(_) => return Err(error::ParseError::InvalidFormat),
        }
    }
    Ok(result)
}

pub fn dump_as_bin<W: io::Write>(
    writer: &mut W,
    transactions: &[Transaction],
) -> Result<(), error::DumpError> {
    for tx in transactions {
        writer.write_all(&tx_to_bin(tx))?;
    }
    Ok(())
}

fn tx_to_bin(tx: &Transaction) -> Vec<u8> {
    let size = calculate_size(tx);
    let mut result = Vec::<u8>::with_capacity(size);
    let raw_header = Header::new(size as u32).dump();
    let raw_tx = dump_tx(tx);

    result.extend_from_slice(&raw_header);
    result.extend_from_slice(&raw_tx);

    result
}

fn calculate_size(tx: &Transaction) -> usize {
    let mut result: usize = 0;

    result += Header::sizeof();
    result += sizeof_tx(tx);
    result += mem::size_of::<u32>(); // DESC_LEN field

    result
}

fn sizeof_tx(tx: &Transaction) -> usize {
    size_of_val(&tx.id)
        + size_of_val(&tx.r#type)
        + size_of_val(&tx.from_user)
        + size_of_val(&tx.to_user)
        + size_of_val(&tx.amount)
        + size_of_val(&tx.timestamp)
        + size_of_val(&tx.status)
        + size_of_val(&tx.description.len())
}

fn dump_tx(tx: &Transaction) -> Vec<u8> {
    let mut res = Vec::<u8>::with_capacity(sizeof_tx(tx));
    res.extend_from_slice(&tx.id.to_be_bytes());
    res.extend_from_slice(&(tx.r#type as u8).to_be_bytes());
    res.extend_from_slice(&tx.from_user.to_be_bytes());
    res.extend_from_slice(&tx.to_user.to_be_bytes());
    res.extend_from_slice(&tx.amount.to_be_bytes());
    res.extend_from_slice(&tx.timestamp.to_be_bytes());
    res.extend_from_slice(&(tx.status as u8).to_be_bytes());
    res.extend_from_slice(&(tx.description.len() as u32).to_be_bytes());
    res.extend_from_slice(tx.description.as_bytes());

    res
}
