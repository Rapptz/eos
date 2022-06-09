# eos-tzdata

This is just a data repository for the underlying IANA [tzdb][tzdb] for use with the `eos-tz` crate's `bundled` feature. It's not meant to be used directly.

## Versioning

The versioning for this repository follows a calendar style versioning system similar to the underlying database itself. The lettering (e.g. `e`) is turned into a digit. Therefore, `2021e` of the IANA database is turned into `2021.5`. Due to Cargo requiring the usage of Semantic Versioning, the library will prefix all releases with a `1`. Therefore the final version number for `2021e` becomes `1.2021.5`.

This has the added benefit of allowing users to just specify `1` as the version to get the latest up to date timezone data.

## Updating

To update, run `python3 update.py` on a Linux-based machine and make a commit. Note that this requires the `requests` module. This script has been modified from the [tzdata][tzdata] repository.

[tzdata]: https://github.com/python/tzdata
