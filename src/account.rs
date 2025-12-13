#[derive(Default, Clone, Copy)]
pub enum AccountStatus {
    #[default]
    Active,
    Locked,
}

#[derive(Default)]
pub struct Account {
    id: u16,
    available: f64,
    held: f64,
    status: AccountStatus
}

impl Account {
    pub fn new(id: u16) -> Self {
        Account {
            id,
            ..Account::default()
        }
    }
}
