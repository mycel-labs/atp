use crate::domain::models::account::Account;

pub trait IAccountRepository {
    fn insert(&self, account: Account) -> Result<Account, String>;
    fn get(&self, id: &str) -> Result<Account, String>;
    fn exists(&self, id: &str) -> bool;
    fn find_by_owner(&self, owner: &str) -> Result<Vec<Account>, String>;
}
