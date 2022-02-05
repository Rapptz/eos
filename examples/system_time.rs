use eos::{DateTime, System};

// This example shows the two ways to get the local time.

fn main() -> Result<(), eos::Error> {
    println!("{}", DateTime::now()?);
    println!("{}", System::now()?);
    Ok(())
}
