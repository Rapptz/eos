## eos

`eos` is a datetime library for Rust, aimed at being robust, simple, and easy to use. `eos` is made with the assumption of operating under a [proleptic Gregorian calendar][greg-cal]. If you're looking for different calendar systems, similar to those in JavaScript's [Temporal library][temporal], then this library is not for you. Although I am open to the idea of having calendar systems in the future.

**Right now, this library is in its development phase.**

### Features

`eos` supports `no_std` targets and some optional features.

**Default features:**

- `alloc`: Enable features that require allocation.
- `std`: Enable features that require the standard library. Implies `alloc`.

### Why not `chrono` or `time`?

There already exist well established libraries in the ecosystem to deal with both dates and times so it's fair to be skeptical of any new library in this space. However, this library was created due to inadequacies in both offerings when it came to more complicated offerings. I had wanted to create a library that was both simpler, more robust, and correct when it came to the complexities of dealing with time.

Timezone naive datetimes are often enough for basic cases but when faced with more complex use cases they often show limitations. Due to this, both `chrono` and `time` can have erradic and surprising behaviour when it comes with working with timezone-aware dates and times. For example, comparisons, hashing, and switching are not timezone aware. `eos` aims to have timezones as a core concept within the library and not as a second thought.

### Design

The design of this library was inspired by the following:

- [Python's `datetime` module][pydt]
- [Java's `java.time`][javadt]
- [Joda-Time][joda-time]
- [Noda-Time][noda-time]

Unlike most datetime libraries, `eos` does not have the concept of a "naive" datetime without a timezone. All datetimes must have a timezone attached to them, a sensible default being UTC or local time. `eos` only supports ISO 8601 dates used throughout the world and is exclusively on the proleptic Gregorian calendar. This makes it not ideal for dates dealing with the past or with alternative Calendar systems. `eos` also assumes that there are 86400 seconds in a day.

[greg-cal]: https://en.wikipedia.org/wiki/Proleptic_Gregorian_calendar
[temporal]: https://github.com/tc39/proposal-temporal
[pydt]: https://docs.python.org/3/library/datetime.html
[javadt]: https://docs.oracle.com/javase/8/docs/api/java/time/package-summary.html
[joda-time]: https://www.joda.org/joda-time/
[noda-time]: https://nodatime.org
