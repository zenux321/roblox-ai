//! Diff pe linii (LCS), suficient pentru fișierele mici ale demo-ului.

use crate::dto::{DiffLine, FileDiff};

/// Peste această dimensiune nu calculăm LCS; afișăm înlocuirea completă.
const MAX_LINES: usize = 3000;

pub fn file_diff(path: &str, before: Option<&str>, after: &str) -> FileDiff {
    let old: Vec<&str> = before.map(|b| b.lines().collect()).unwrap_or_default();
    let new: Vec<&str> = after.lines().collect();
    let lines = if old.len() > MAX_LINES || new.len() > MAX_LINES {
        old.iter()
            .map(|l| line("del", None, l))
            .chain(
                new.iter()
                    .enumerate()
                    .map(|(i, l)| line("add", Some(i + 1), l)),
            )
            .collect()
    } else {
        lcs_diff(&old, &new)
    };
    FileDiff {
        path: path.into(),
        added: lines.iter().filter(|l| l.kind == "add").count(),
        removed: lines.iter().filter(|l| l.kind == "del").count(),
        is_new: before.is_none(),
        lines,
    }
}

fn line(kind: &str, number: Option<usize>, text: &str) -> DiffLine {
    DiffLine {
        kind: kind.into(),
        number,
        text: text.into(),
    }
}

fn lcs_diff(old: &[&str], new: &[&str]) -> Vec<DiffLine> {
    let (n, m) = (old.len(), new.len());
    let mut table = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            table[i][j] = if old[i] == new[j] {
                table[i + 1][j + 1] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }
    let (mut i, mut j, mut out) = (0, 0, Vec::new());
    while i < n || j < m {
        if i < n && j < m && old[i] == new[j] {
            out.push(line("ctx", Some(j + 1), new[j]));
            i += 1;
            j += 1;
        } else if j < m && (i == n || table[i][j + 1] >= table[i + 1][j]) {
            out.push(line("add", Some(j + 1), new[j]));
            j += 1;
        } else {
            out.push(line("del", None, old[i]));
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_file_is_all_additions() {
        let d = file_diff("a.luau", None, "a\nb\n");
        assert!(d.is_new);
        assert_eq!((d.added, d.removed), (2, 0));
    }

    #[test]
    fn changed_line_is_one_deletion_and_one_addition() {
        let d = file_diff("a.luau", Some("x\ny\nz"), "x\nY\nz");
        assert_eq!((d.added, d.removed), (1, 1));
        let kinds: Vec<&str> = d.lines.iter().map(|l| l.kind.as_str()).collect();
        assert_eq!(kinds, vec!["ctx", "add", "del", "ctx"]);
        assert_eq!(d.lines[1].number, Some(2));
    }
}
