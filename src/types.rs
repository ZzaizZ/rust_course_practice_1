type TxId = u64;
type UserId = u64;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TxType {
    #[default]
    Unknown,
    Deposit,
    Transfer,
    Withdrawal,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TxStatus {
    #[default]
    Unknown,
    Success,
    Failure,
    Pending,
}

#[derive(Debug, Default, PartialEq)]
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
