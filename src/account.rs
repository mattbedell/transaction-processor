pub enum AccountStatus {
    Active,
    Locked,
}

pub struct Account {
    id: u16,
    available: f64,
    held: f64,
    total: f64,
    status: AccountStatus
}
