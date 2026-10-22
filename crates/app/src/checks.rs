//! Verificări statice deterministe pe proiectul demonstrativ.

use std::fs;
use std::path::Path;

use rab_core::paths::{RelPath, parse_relative};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckOutcome {
    pub passed: bool,
    pub note: String,
}

/// `default.project.json` are numele, arborele DataModel și căi existente în proiect.
pub fn rojo_structure(root: &Path) -> CheckOutcome {
    let text = match fs::read_to_string(root.join("default.project.json")) {
        Ok(t) => t,
        Err(_) => return fail("default.project.json lipsește"),
    };
    let value: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => return fail(&format!("JSON invalid: {e}")),
    };
    if !value["name"].is_string() {
        return fail("lipsește câmpul `name`");
    }
    if value["tree"]["$className"] != "DataModel" {
        return fail("rădăcina `tree` trebuie să fie DataModel");
    }
    let mut paths = Vec::new();
    collect_paths(&value["tree"], &mut paths);
    for p in &paths {
        match parse_relative(p) {
            Ok(rel) if root.join(rel.as_string()).exists() => {}
            Ok(_) => return fail(&format!("`$path` {p} nu există")),
            Err(e) => return fail(&format!("`$path` {p}: {e}")),
        }
    }
    pass(&format!("{} căi mapate, toate în proiect", paths.len()))
}

fn collect_paths(node: &serde_json::Value, out: &mut Vec<String>) {
    if let Some(map) = node.as_object() {
        for (key, child) in map {
            if key == "$path" {
                if let Some(p) = child.as_str() {
                    out.push(p.to_string());
                }
            } else if !key.starts_with('$') {
                collect_paths(child, out);
            }
        }
    }
}

/// Niciun `require(<număr>)` (module din Creator Store încărcate după ID).
pub fn no_external_require(root: &Path) -> CheckOutcome {
    let mut offenders = Vec::new();
    let mut scanned = 0;
    visit_luau(&root.join("src"), &mut |path, text| {
        scanned += 1;
        if has_numeric_require(text) {
            offenders.push(
                path.strip_prefix(root)
                    .unwrap_or(path)
                    .display()
                    .to_string(),
            );
        }
    });
    if offenders.is_empty() {
        pass(&format!("{scanned} fișiere Luau verificate"))
    } else {
        fail(&format!("require după ID în: {}", offenders.join(", ")))
    }
}

fn has_numeric_require(text: &str) -> bool {
    let mut rest = text;
    while let Some(i) = rest.find("require") {
        let after = rest[i + "require".len()..].trim_start();
        let after = after.strip_prefix('(').map(str::trim_start).unwrap_or("");
        if after.starts_with(|c: char| c.is_ascii_digit()) || after.starts_with("\"rbxassetid") {
            return true;
        }
        rest = &rest[i + "require".len()..];
    }
    false
}

fn visit_luau(dir: &Path, f: &mut dyn FnMut(&Path, &str)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            visit_luau(&path, f);
        } else if path.extension().is_some_and(|e| e == "luau" || e == "lua")
            && let Ok(text) = fs::read_to_string(&path)
        {
            f(&path, &text);
        }
    }
}

/// Toate fișierele scrise în rulare sunt în căile de scriere ale planului.
pub fn changes_in_scope(changed: &[String], scopes: &[RelPath]) -> CheckOutcome {
    let outside: Vec<&String> = changed
        .iter()
        .filter(|p| parse_relative(p).map_or(true, |rel| !scopes.iter().any(|s| rel.is_within(s))))
        .collect();
    if outside.is_empty() {
        pass(&format!(
            "{} fișiere modificate, toate în căile aprobate",
            changed.len()
        ))
    } else {
        fail(&format!(
            "în afara planului: {}",
            outside
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

fn pass(note: &str) -> CheckOutcome {
    CheckOutcome {
        passed: true,
        note: note.into(),
    }
}

fn fail(note: &str) -> CheckOutcome {
    CheckOutcome {
        passed: false,
        note: note.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_require_is_detected() {
        assert!(has_numeric_require("local m = require(123456)"));
        assert!(has_numeric_require("require ( 42 )"));
        assert!(has_numeric_require("require(\"rbxassetid://1\")"));
        assert!(!has_numeric_require(
            "local c = require(ReplicatedStorage.Shared.Config)"
        ));
    }
}
