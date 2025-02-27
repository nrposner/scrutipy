pub mod rounding;
pub mod sd_binary;
pub mod utils;
use std::env;

use utils::*;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    //let res = decimal_places_scalar(Some("8.78"), ".").unwrap_or(0);

    let mean = 0.3;
    let n = 30;
    let res = (((n - (n - 1)) as f64) * (mean * (1.0 - mean))).sqrt();
    println!("{}", res);
}
