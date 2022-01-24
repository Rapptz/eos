use eos::{Local, format_dt};

// Example showcasing different forms of formatting

fn main() -> Result<(), eos::Error> {
    // ISO formatting
    let now = Local::now()?;
    println!("{}", now);
    // Manual formatting
    println!("{}", format_dt!("%A, %d %B %Y %I:%M:%S %p %Z", now));
    Ok(())
}
