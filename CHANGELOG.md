v0.6.0 (in development)
-----------------------
- Replaced `futures` and `tokio-stream` dependencies with `futures-core` and
  `futures-sink`.  This is more lightweight and should fix a problem with the
  `Sink` and `Stream` traits not being hyperlinked on docs.rs.

v0.5.0 (2023-11-22)
-------------------
- Added `asynchronous` category to crate metadata
- Increased MSRV to 1.70
- Removed the implicit features for the optional dependencies that comprise the
  `async` feature
- Remove library installation instructions from README
- Derive `Clone`, `Eq`, and `PartialEq` for `JsonLinesReader`,
  `JsonLinesWriter`, `JsonLinesIter`, `AsyncJsonLinesReader`,
  `AsyncJsonLinesWriter`, and `JsonLinesSink`

v0.4.0 (2022-10-31)
-------------------
- Added tokio-based asynchronous functionality:
    - `AsyncJsonLinesReader`
    - `AsyncJsonLinesWriter`
    - `AsyncBufReadJsonLines` extension trait
    - `AsyncWriteJsonLines` extension trait

v0.3.0 (2022-10-30)
-------------------
- Renamed `JsonLinesReader::iter()` to `read_all()`
- Renamed `Iter` to `JsonLinesIter`, and renamed `JsonLinesIter` to
  `JsonLinesFileIter`

v0.2.0 (2022-10-29)
-------------------
- Gave `JsonLinesReader` and `JsonLinesWriter` new `get_ref()` and `get_mut()`
  methods

v0.1.0 (2022-10-28)
-------------------
Initial release
