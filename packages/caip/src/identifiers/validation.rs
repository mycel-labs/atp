use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // CAIP-2: namespace:reference
    pub static ref CHAIN_ID_REGEX: Regex = Regex::new(r"^([-a-z0-9]{3,8}):([-_a-zA-Z0-9]{1,32})$").unwrap();

    // CAIP-10: chain_id:account_address
    pub static ref ACCOUNT_ID_REGEX: Regex = Regex::new(r"^([-a-z0-9]{3,8}):([-_a-zA-Z0-9]{1,32}):([-a-zA-Z0-9]{1,128})$").unwrap();

    // CAIP-19: chain_id/namespace:reference
    pub static ref ASSET_ID_REGEX: Regex = Regex::new(r"^([-a-z0-9]{3,8}):([-a_-zA-Z0-9]{1,32})/([-a-z0-9]{3,8}):([-a-zA-Z0-9]{1,64})$").unwrap();

    // CAIP-19: namespace:reference
    pub static ref ASSET_ID_BASE_REGEX: Regex = Regex::new(r"^([-a-z0-9]{3,8}):([-a-zA-Z0-9]{1,64})$").unwrap();
}
