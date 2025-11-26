type TxId = u64;
type UserId = u64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TxType {
    Deposit,
    Transfer,
    Withdrawal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TxStatus {
    Success,
    Failure,
    Pending,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
    pub id: TxId,
    pub r#type: TxType,
    pub from_user: UserId,
    pub to_user: UserId,
    pub amount: u64,
    pub timestamp: u64,
    pub status: TxStatus,
    pub description: String,
}
