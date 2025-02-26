pub mod rounding;
pub mod sd_binary;
pub mod utils;
use std::env;

use utils::*;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let res = decimal_places_scalar(Some("8.78"), ".").unwrap_or(0);

    println!("{}", res);
}
