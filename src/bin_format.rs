use crate::error;
use std::{
    io::{self, Cursor},
    mem,
};

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
    let Ok(s) = String::from_utf8(buf) else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"));
    };
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

fn read_tx(
    reader: &mut impl io::Read,
    full_record_size: u32,
) -> Result<Transaction, error::ParseError> {
    let id = read_u64(reader)?;
    let r#type = read_tx_type(reader)?;
    let from_user = read_u64(reader)?;
    let to_user = read_u64(reader)?;
    let amount = read_u64(reader)?;
    let timestamp = read_u64(reader)?;
    let status = read_tx_status(reader)?;
    let desc_len = read_u32(reader)?;

    if full_record_size != MIN_RECORD_SIZE + desc_len {
        return Err(error::ParseError::InvalidFormat(
            "mailformed record. record size mismatch".to_string(),
        ));
    }

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

/// минимально возможный размер записи без описания
const MIN_RECORD_SIZE: u32 = 46;

/// Читает и парсит транзакции из бинарного формата.
///
/// # Аргументы
///
/// * `reader` - Источник данных. Это может быть открытый файл, сетевой поток или
///   массив байт. Должен реализовывать трейт [`std::io::Read`].  
///   Данные должны быть в текстовом формате ([doc/YPBankBinFormat_ru.md](doc/YPBankBinFormat_ru.md))
///
/// # Ошибки
///
/// Возвращает [`ParseError`], если:
/// * Формат данных некорректен.
/// * Возникла ошибка ввода-вывода при чтении из `reader`.
///
/// # Пример
///
/// Чтение из массива байт:
///
/// ```rust
/// use ypbank_parser::{parse_from_bin, types::Transaction};
///
/// let mut data: &[u8] = &[
///            0x59, 0x50, 0x42, 0x4e,
///            0x00, 0x00, 0x00, 0x32,
///            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
///            0x00,
///            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
///            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
///            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
///            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
///            0x00,
///            0x00, 0x00, 0x00, 0x04,
///            0x74, 0x65, 0x73, 0x74,
///        ];
///
/// let txs = parse_from_bin(&mut data).expect("Ошибка парсинга");
/// ```
pub fn parse_from_bin(reader: &mut impl io::Read) -> Result<Vec<Transaction>, error::ParseError> {
    let mut result = Vec::<Transaction>::new();
    loop {
        match Header::read(reader) {
            Ok(header) => {
                if header.record_size < MIN_RECORD_SIZE {
                    return Err(error::ParseError::InvalidFormat(
                        "mailformed record. record size too small".to_string(),
                    ));
                }
                let mut buf = vec![0u8; header.record_size as usize];
                reader.read_exact(&mut buf)?;
                let mut buffer_reader = Cursor::new(buf);
                let tx = read_tx(&mut buffer_reader, header.record_size)?;
                result.push(tx);
            }
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(err) => return Err(error::ParseError::InvalidFormat(err.to_string())),
        }
    }
    Ok(result)
}

/// Сериализует список транзакций в бинарный формат, записывая результат во `writer`.
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
/// use ypbank_parser::{dump_as_bin, types::{Transaction, TxStatus, TxType}, };
///
/// let txs = vec![Transaction{id: 1, r#type: TxType::Deposit,
///                            from_user: 1001, to_user: 1001,
///                            amount: 1001, timestamp: 1633036800000,
///                            status: TxStatus::Success,
///                            description: "Description".to_string()}];
/// let mut buffer: Vec<u8> = Vec::new();
///
/// dump_as_bin(&mut buffer, &txs).expect("Ошибка записи");
///
/// let magic_number: &[u8] = &[0x59u8, 0x50u8, 0x42u8, 0x4eu8];
/// assert!(buffer.starts_with(magic_number));
/// ```
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
    let tx_bytes_size = calculate_size(tx);
    let mut result = Vec::<u8>::with_capacity(tx_bytes_size);
    let raw_header = Header::new(tx_bytes_size as u32).dump();
    let raw_tx = dump_tx(tx);

    result.extend_from_slice(&raw_header);
    result.extend_from_slice(&raw_tx);

    result
}

fn calculate_size(tx: &Transaction) -> usize {
    let mut result: usize = 0;

    result += sizeof_tx(tx);
    result += mem::size_of::<u32>(); // DESC_LEN field

    result
}

fn sizeof_tx(tx: &Transaction) -> usize {
    size_of_val(&tx.id)
        + size_of_val(&(tx.r#type as u8))
        + size_of_val(&tx.from_user)
        + size_of_val(&tx.to_user)
        + size_of_val(&tx.amount)
        + size_of_val(&tx.timestamp)
        + size_of_val(&(tx.status as u8))
        + tx.description.len()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dump_header() {
        let header = Header::new(10);

        #[rustfmt::skip]
        let expected_bytes: [u8; 8] = [
            0x59, 0x50, 0x42, 0x4e,
            0x00, 0x00, 0x00, 0x0A
        ];

        let got = header.dump();

        assert_eq!(got.len(), 8);

        assert_eq!(&expected_bytes[..], &got[..]);
    }

    #[test]
    fn test_dump_tx() {
        let tx = Transaction {
            id: 1001,
            r#type: TxType::Deposit,
            from_user: 1001,
            to_user: 0,
            amount: 1001,
            timestamp: 1001,
            status: TxStatus::Success,
            description: "test".to_string(),
        };

        #[rustfmt::skip]
        let expected: [u8; 50] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00,
            0x00, 0x00, 0x00, 0x04,
            0x74, 0x65, 0x73, 0x74,
        ];

        let got = dump_tx(&tx);

        assert_eq!(expected[..], got[..]);
    }

    #[test]
    fn test_calculate_size() {
        let tx = Transaction {
            id: 1001,
            r#type: TxType::Deposit,
            from_user: 1001,
            to_user: 0,
            amount: 1001,
            timestamp: 1001,
            status: TxStatus::Success,
            description: "test".to_string(),
        };

        let expected = 50;

        let got = calculate_size(&tx);

        assert_eq!(expected, got);
    }

    #[test]
    fn test_parse_from_bin() {
        #[rustfmt::skip]
        let mut data: &[u8] = &[
            0x59, 0x50, 0x42, 0x4e,
            0x00, 0x00, 0x00, 0x32,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00,
            0x00, 0x00, 0x00, 0x04,
            0x74, 0x65, 0x73, 0x74,
        ];

        let expected = Transaction {
            id: 1001,
            r#type: TxType::Deposit,
            from_user: 1001,
            to_user: 0,
            amount: 1001,
            timestamp: 1001,
            status: TxStatus::Success,
            description: "test".to_string(),
        };

        let got = parse_from_bin(&mut data);

        assert!(got.is_ok());
        assert_eq!(got.as_ref().unwrap().len(), 1);
        assert_eq!(expected, got.as_ref().unwrap()[0]);
    }

    #[test]
    fn test_parse_mailformed_record() {
        #[rustfmt::skip]
        let mut data: &[u8] = &[
            0x59, 0x50, 0x42, 0x4e,
            0x00, 0x00, 0x00, 0x10, // запись слишком маленькая
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00,
            0x00, 0x00, 0x00, 0x04,
            0x74, 0x65, 0x73, 0x74,
        ];

        let got = parse_from_bin(&mut data);

        assert!(got.is_err());
    }

    #[test]
    fn test_mismatch_record_size() {
        #[rustfmt::skip]
        let mut data: &[u8] = &[
            0x59, 0x50, 0x42, 0x4e,
            0x00, 0x00, 0x00, 0x32,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xe9,
            0x00,
            0x00, 0x00, 0x00, 0x05, // описание длиной 5, а не 4
            0x74, 0x65, 0x73, 0x74,
        ];

        let got = parse_from_bin(&mut data);

        assert!(got.is_err());
    }
}
