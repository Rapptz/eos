#[cfg(feature = "bundled")]
use eos_tz::zone;

#[cfg(feature = "bundled")]
fn main() {
    let now = eos::Utc::now();
    println!("{}", &now);
    let zone = zone!("America/New_York");
    println!("{}", now.in_timezone(zone));
}

#[cfg(not(feature = "bundled"))]
fn main() {}
