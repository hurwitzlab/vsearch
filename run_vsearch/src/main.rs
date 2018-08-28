extern crate run_vsearch;

use std::process;

fn main() {
    let config = run_vsearch::get_args().expect("Could not get arguments");

    if let Err(e) = run_vsearch::run(config) {
        println!("Error: {}", e);
        process::exit(1);
    }
}
