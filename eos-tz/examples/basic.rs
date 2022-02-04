use eos::{datetime, Utc};

// A basic example showing how to use the timezone data.

fn main() -> Result<(), eos_tz::Error> {
    // Retrieve today's date in UTC:
    let today = Utc::now();

    // Get the timezone in New York
    let ny = eos_tz::TimeZone::get("America/New_York")?;

    // See the date in new york
    println!("{}", today.in_timezone(ny));

    // Check the date in Samoa in December 30th 2011
    let dec30 = datetime!(2011-12-30 1:30);
    let samoa = eos_tz::TimeZone::get("Pacific/Apia")?;

    // They actually skipped this entire day due to a timezone transition.
    // Note `with_timezone` uses the same local time (2011-12-30 1:30) rather than
    // assuming it's UTC.
    println!("{}", dec30.with_timezone(samoa));
    Ok(())
}
