# eos-tz

This is a companion library that offers support for the [IANA database][tzdb], colloquially also known as `zoneinfo`, `tzdb`, or just `tz`.

Most UNIX-like operating systems come with their own copy of the `tzdb`, it's for this reason that the library has first class support for using the system provided IANA database which should make for a more seemless integration that is always up to date for applications that need it.

For Windows users, the situation is a bit different. There are two approaches that can be done to load compiled `TZif` files. The first is through usage of the `eos_tz::TimeZone::load` method with a specified path that has the compiled data. The other is to use the `bundled` feature which embeds the ~1.8 MiB timezone information directly into the executable. Note that if either of these approaches are taken then you become responsible for keeping the timezone data up to date and correct.

In the future it may be possible to directly load timezone information from the operating system though right now this library only works with the IANA database.

## Features

Unlike the parent library, this doesn't work in `no_std` targets. Therefore there is no feature to disable the `std` integration. This is due to the usage of File I/O and allocation. Likewise, this library uses the default set of features of the parent library without a way to customise it. This includes features such as `macros` or `localtime`.

- `bundled`: Bundle the data of the `tzdb` at compile time directly into the executable. This bundles the data from the `eos-tzdata` crate.

[tzdb]: https://www.iana.org/time-zones
