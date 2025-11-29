# YPBank_parser

Библиотека для парсинга, сериализации и конвертации истории транзакций между форматами CSV, BIN, и TEXT.

Запуск бинарников
Для проверки функциональности используются команды `cargo run --bin <имя_бинарника> -- <аргументы>`.

## ypbank_comparer
Сравнивает две истории транзакций из указанных файлов и форматов. Выведет первую несовпавшую транзакцию в паре файлов. Форматы файлов могут быть разные.

```bash
cargo run --bin ypbank_comparer -- \
    --file1 example_data/transactions.csv \
    --format1 csv \
    --file2 example_data/another_transactions_4.csv \
    --format2 csv
```

Ожидаемый вывод:

```
Наборы транзакций не иднетичны!
Несовпали транзакции на позииции 4
LHS:
None

RHS:
Some(
    Transaction {
        id: 1004,
        type: Transfer,
        from_user: 501,
        to_user: 502,
        amount: 15000,
        timestamp: 1672534800000,
        status: Failure,
        description: "Payment for services, invoice #123",
    },
)
```

## ypbank_converter
Читает данные из входного файла и конвертирует их в указанный выходной формат.

Пример (BIN в TEXT, вывод в stdout):

```bash
cargo run --bin ypbank_converter -- \
    --input-file example_data/transactions.bin \
    --input-format bin \
    --output-format text
```

Ожидаемый вывод:

```
TIMESTAMP: 1672531200000
TX_ID: 1001
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 501
AMOUNT: 50000
STATUS: SUCCESS
DESCRIPTION: "Initial account funding"

DESCRIPTION: "Payment for services, invoice #123"
TX_ID: 1002
TO_USER_ID: 502
TX_TYPE: TRANSFER
FROM_USER_ID: 501
AMOUNT: 15000
TIMESTAMP: 1672534800000
STATUS: FAILURE

TX_TYPE: WITHDRAWAL
TO_USER_ID: 0
STATUS: PENDING
TX_ID: 1003
AMOUNT: 1000
DESCRIPTION: "ATM withdrawal"
TIMESTAMP: 1672538400000
FROM_USER_ID: 502
```