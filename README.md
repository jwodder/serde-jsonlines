[![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)
[![CI Status](https://github.com/jwodder/serde-jsonlines/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/serde-jsonlines/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/serde-jsonlines/branch/master/graph/badge.svg)](https://codecov.io/gh/jwodder/serde-jsonlines)
[![MIT License](https://img.shields.io/github/license/jwodder/serde-jsonlines.svg)](https://opensource.org/licenses/MIT)

[GitHub](https://github.com/jwodder/serde-jsonlines) | [crates.io](https://crates.io/crates/serde-jsonlines) | [Documentation](https://docs.rs/serde-jsonlines) | [Issues](https://github.com/jwodder/serde-jsonlines/issues) | [Changelog](https://github.com/jwodder/serde-jsonlines/blob/master/CHANGELOG.md)

JSON Lines (a.k.a. newline-delimited JSON) is a simple format for storing
sequences of JSON values in which each value is serialized on a single line and
terminated by a newline sequence.  The `serde-jsonlines` crate provides
functionality for reading & writing these documents (whether all at once or
line by line) using `serde`'s serialization & deserialization features.

Basic usage involves simply importing the `BufReadExt` or `WriteExt` extension
trait and then using the `json_lines()` or `write_json_lines()` method on a
`BufRead` or `Write` value to read or write a sequence of JSON Lines values.
Convenience functions are also provided for the common case of reading or
writing a JSON Lines file given as a filepath.

At a lower level, values can be read or written one at a time (which is useful
if, say, different lines are different types) by wrapping a `BufRead` or
`Write` value in a `JsonLinesReader` or `JsonLinesWriter` and then calling the
wrapped structure's `read()` or `write()` method, respectively.

Installation
============

`serde-jsonlines` requires version 1.56 of Rust or higher.  To use the
`serde-jsonlines` library in your Cargo project, add the following to your
`Cargo.toml`:

```toml
[dependencies]
serde-jsonlines = "0.3.0"
```


Example
=======

```rust
use serde::{Deserialize, Serialize};
use serde_jsonlines::{json_lines, write_json_lines};
use std::io::Result;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Structure {
    pub name: String,
    pub size: i32,
    pub on: bool,
}

fn main() -> Result<()> {
    let values = vec![
        Structure {
            name: "Foo Bar".into(),
            size: 42,
            on: true,
        },
        Structure {
            name: "Quux".into(),
            size: 23,
            on: false,
        },
        Structure {
            name: "Gnusto Cleesh".into(),
            size: 17,
            on: true,
        },
    ];
    write_json_lines("example.jsonl", &values)?;
    let values2 = json_lines("example.jsonl")?.collect::<Result<Vec<Structure>>>()?;
    assert_eq!(values, values2);
    Ok(())
}
```
