## eos

`eos` is a datetime library for Rust, aimed at being robust, simple, and easy to use. `eos` is made with the assumption of operating under a [proleptic Gregorian calendar][greg-cal]. If you're looking for different calendar systems, similar to those in JavaScript's [Temporal library][temporal], then this library is not for you. However, I am open to the idea of having calendar systems in the future.

**Right now, this library is in its development phase.**

### Why?

There already exist well established libraries in the ecosystem to deal with both dates and times so it's fair to be skeptical of any new library in this space. However, this library was created due to inadequacies in current offerings when it came to more complicated use cases (such as timezones). I had wanted to create a library that was both simpler, more robust, and correct when it came to the complexities of dealing with time.

Special care has been taken to ensure timezones are implemented properly. To that end, there is no concept of a naive date time. The default timezone of a `DateTime` type is UTC. All operations done on a `DateTime` are timezone aware. For example, comparisons are done by comparing the same instant of time in UTC or within the same timezone. Despite having timezone support, the `chrono` crate [does not do this][chrono-cmp]. Which can lead to surprising behaviour.

Since timezone information can be a bit heavy on resources and not something every applicationw ants to concern itself with, the IANA database backed `TimeZone` implementation is in another crate, [`eos-tz`][eos-tz]. The base library only has `Utc`, `UtcOffset`, and `Local` for their concrete timezone implementations.

### Features

`eos` supports `no_std` targets and some optional features.

**Default features:**

- `alloc`: Enable features that require allocation.
- `macros`: Enables the compile-time construction macros. Most of these use `macro_rules!` rather than the proc-macro machinery to keep compile times sane. Unfortunately, due to limitations in `const fn`, the `format_spec!` macro uses proc-macro machinery. To keep compile-times sane for this macro, `syn` and `quote` are **not** used.
- `std`: Enable features that require the standard library. Implies `alloc`.
- `localtime`: Enable features that allow retrieving local time information. Requires `libc` on POSIX.
- `formatting`: Enable features relating to formatting various types. Implies `alloc`.
- `parsing`: Enable features relating to parsing strings to various types. Implies `alloc`.

**Optional features:**

- [`serde`](https://serde.rs): Enable custom Serialize/Deserialize implementations. Requires `parsing` as well.

### Acknowledgements

The design of this library was inspired by the following:

- [Python's `datetime` module][pydt]
- [`arrow`][pyarrow]
- [`dateutil`][dateutil]
- [Java's `java.time`][javadt]
- [Joda-Time][joda-time]
- [Noda-Time][noda-time]
- [Howard Hinnant's `date`][cpp-date]

Certain algorithms come from one of these libraries above. Likewise, due to the difficult nature of testing datetimes, certain tests were adapted from one of these libraries as well to have better test coverage.

Without these libraries, this one would not be possible. `eos` stands on the shoulder of giants.

### License

This project is licensed under the [Apache-2 license][apache].

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `eos` by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.

[greg-cal]: https://en.wikipedia.org/wiki/Proleptic_Gregorian_calendar
[temporal]: https://github.com/tc39/proposal-temporal
[pydt]: https://docs.python.org/3/library/datetime.html
[javadt]: https://docs.oracle.com/javase/8/docs/api/java/time/package-summary.html
[joda-time]: https://www.joda.org/joda-time/
[noda-time]: https://nodatime.org
[cpp-date]: https://github.com/HowardHinnant/date
[pyarrow]: https://github.com/arrow-py/arrow
[dateutil]: https://github.com/dateutil/dateutil
[apache]: https://github.com/Rapptz/eos/blob/master/LICENSE
[chrono-cmp]: https://github.com/chronotope/chrono/blob/f6bd567bb677262645c1fc3131c8c1071cd77ec3/src/datetime.rs#L801-L811
[eos-tz]: https://github.com/Rapptz/eos/tree/master/eos-tz
