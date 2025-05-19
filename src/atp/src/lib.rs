pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod utils;

#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
