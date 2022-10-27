use assert_fs::assert::PathAssert;
use assert_fs::NamedTempFile;
use jsonlines::JsonLinesWriter;
use serde::Serialize;
use std::fs::File;

#[derive(Serialize)]
struct Structure {
    name: String,
    size: i32,
    on: bool,
}

#[test]
fn test_write_one() {
    let tmpfile = NamedTempFile::new("test_write_one.jsonl").unwrap();
    {
        let fp = File::create(&tmpfile).unwrap();
        let mut writer = JsonLinesWriter::new(fp);
        writer
            .write(&Structure {
                name: "Foo Bar".into(),
                size: 42,
                on: true,
            })
            .unwrap();
    }
    tmpfile.assert("{\"name\":\"Foo Bar\",\"size\":42,\"on\":true}\n");
}
