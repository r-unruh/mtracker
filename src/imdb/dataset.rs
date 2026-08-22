//! Streaming reader for the gzipped TSV datasets.

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{anyhow, Context, Result};
use flate2::read::MultiGzDecoder;

/// Split a line into fields (`\N` becomes ""); `None` on a wrong column count
pub fn fields(line: &str, expected: usize) -> Option<Vec<&str>> {
    let fields: Vec<&str> = line.split('\t').map(|v| if v == "\\N" { "" } else { v }).collect();
    (fields.len() == expected).then_some(fields)
}

/// Stream the data lines of a gzipped TSV file; fails loudly if the header changed
pub fn for_each_line(
    path: &Path,
    expected_header: &[&str],
    mut f: impl FnMut(&str) -> Result<()>,
) -> Result<()> {
    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let hint = "Try re-downloading with 'mtracker sync --download'.";

    let file = File::open(path).with_context(|| format!("failed to open {name}"))?;
    let mut reader = BufReader::with_capacity(1 << 20, MultiGzDecoder::new(file));

    let mut line = String::new();
    reader
        .read_line(&mut line)
        .with_context(|| format!("failed to read {name}: corrupt download?\n{hint}"))?;
    let header: Vec<&str> = line.trim_end().split('\t').collect();
    if header != expected_header {
        return Err(anyhow!(
            "unexpected columns in {name}\nexpected: {}\nfound:    {}\nIMDb may have changed the dataset format.",
            expected_header.join(", "),
            header.join(", ")
        ));
    }

    loop {
        line.clear();
        let n = reader
            .read_line(&mut line)
            .with_context(|| format!("failed to read {name}: corrupt download?\n{hint}"))?;
        if n == 0 {
            break;
        }
        f(line.trim_end_matches(['\n', '\r']))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::{write::GzEncoder, Compression};

    use super::*;

    fn write_gz(name: &str, content: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("mtracker_test_{name}.tsv.gz"));
        let mut enc = GzEncoder::new(File::create(&path).unwrap(), Compression::fast());
        enc.write_all(content.as_bytes()).unwrap();
        enc.finish().unwrap();
        path
    }

    #[test]
    fn parses_rows_and_nulls() {
        let path = write_gz("rows", "a\tb\tc\n1\t\\N\tx\nbad row\n2\ty\t\\N\n");
        let mut rows = vec![];
        let mut skipped = 0;
        for_each_line(&path, &["a", "b", "c"], |line| {
            match fields(line, 3) {
                Some(f) => rows.push(f.iter().map(|s| s.to_string()).collect::<Vec<_>>()),
                None => skipped += 1,
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(rows, vec![vec!["1", "", "x"], vec!["2", "y", ""]]);
        assert_eq!(skipped, 1);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn rejects_unexpected_header() {
        let path = write_gz("header", "a\tb\n1\t2\n");
        let err = for_each_line(&path, &["a", "c"], |_| Ok(())).unwrap_err();
        assert!(err.to_string().starts_with("unexpected columns"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn rejects_corrupt_file() {
        let mut path = std::env::temp_dir();
        path.push("mtracker_test_corrupt.tsv.gz");
        std::fs::write(&path, b"this is not gzip").unwrap();
        let err = for_each_line(&path, &["a"], |_| Ok(())).unwrap_err();
        assert!(err.to_string().contains("corrupt"));
        std::fs::remove_file(path).ok();
    }
}
