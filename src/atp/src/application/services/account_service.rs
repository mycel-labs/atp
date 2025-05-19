use crate::infrastructure::repositories::account_repository_impl::AccountRepositoryImpl;
use crate::infrastructure::repositories::signer_repository_impl::SignerRepositoryImpl;

pub struct AccountService {
    account_repository: AccountRepositoryImpl,
    signer_repository: SignerRepositoryImpl,
}

impl AccountService {
    pub fn new(
        account_repository: AccountRepositoryImpl,
        signer_repository: SignerRepositoryImpl,
    ) -> Self {
        Self {
            account_repository,
            signer_repository,
        }
    }
}
