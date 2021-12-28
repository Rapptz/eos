use eos::{DateTime, Utc};

// This example shows the two ways to get UTC time.

fn main() {
    println!("{:?}", DateTime::utc_now());
    println!("{:?}", Utc::now());
}
