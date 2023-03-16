v0.5.0 (in development)
-----------------------
- Added `asynchronous` category to crate metadata
- Increased MSRV to 1.64
- Removed the implicit features for the optional dependencies that comprise the
  `async` feature

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
