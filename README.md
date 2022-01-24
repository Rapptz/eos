## eos

`eos` is a datetime library for Rust, aimed at being robust, simple, and easy to use. `eos` is made with the assumption of operating under a [proleptic Gregorian calendar][greg-cal]. If you're looking for different calendar systems, similar to those in JavaScript's [Temporal library][temporal], then this library is not for you. However, I am open to the idea of having calendar systems in the future.

**Right now, this library is in its development phase.**

### Features

`eos` supports `no_std` targets and some optional features.

**Default features:**

- `alloc`: Enable features that require allocation.
- `macros`: Enables the compile-time construction macros. These use `macro_rules!` rather than the proc-macro machinery to keep compile times sane.
- `std`: Enable features that require the standard library. Implies `alloc`.
- `localtime`: Enable features that allow retrieving local time information. Requires `libc` on POSIX.
- `format`: Enable features relating to formatting various types. This also adds the The `format_spec!` macro which uses proc-macros due to limitations in `const fn`. Implies `alloc`.

### Why not `chrono` or `time`?

There already exist well established libraries in the ecosystem to deal with both dates and times so it's fair to be skeptical of any new library in this space. However, this library was created due to inadequacies in both offerings when it came to more complicated use cases (such as timezones). I had wanted to create a library that was both simpler, more robust, and correct when it came to the complexities of dealing with time.

Timezone naive datetimes are often enough for basic cases but when faced with more complex use cases they often show limitations. Due to this, both `chrono` and `time` can have erratic and surprising behaviour when it comes with working with timezone-aware dates and times. For example, comparisons, hashing, and switching are not timezone aware. `eos` aims to have timezones as a core concept within the library and not as a second thought.

### Design

Unlike most datetime libraries, `eos` does not have the concept of a "naive" datetime without a timezone. All datetimes must have a timezone attached to them, a sensible default being UTC or local time. `eos` only supports ISO 8601 dates used throughout the world and is exclusively on the proleptic Gregorian calendar. This makes it not ideal for dates dealing with the far past or with alternative calendar systems. `eos` also assumes that there are 86400 seconds in a day.

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
