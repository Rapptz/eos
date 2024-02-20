use eos_tz::Local;

// This example shows how to get the local time.

fn main() -> Result<(), eos_tz::Error> {
    println!("{}", Local::now()?);
    println!("{}", eos::now_in(Local::new()?));
    Ok(())
}
