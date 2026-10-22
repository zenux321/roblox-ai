//! Contractele de configurare și validarea lor.
//!
//! Parsarea respinge câmpurile necunoscute (`deny_unknown_fields`). Validarea
//! semantică are două moduri: `Contract` verifică invariantele de siguranță
//! (exemplele din `config/` trebuie să treacă), iar `Execution` adaugă cerințele
//! pentru o configurație care pornește lucrul real: fără `example_only`, fără
//! placeholder-e și fără funcții încă neimplementate.

pub mod agents;
pub mod collaboration;

use serde::Serialize;
use thiserror::Error;

pub use agents::AgentsConfig;
pub use collaboration::CollaborationConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Contractul este coerent și sigur; poate fi încă exemplu.
    Contract,
    /// Configurația va fi folosită pentru execuție.
    Execution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCode {
    UnsupportedSchemaVersion,
    ExampleOnly,
    Placeholder,
    UnsafeFlag,
    InvalidValue,
    MissingValue,
    ProviderDisabled,
    DuplicateId,
    AgentToolNotAllowed,
    ContractMismatch,
    Inconsistent,
    FeatureNotAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issue {
    pub path: String,
    pub code: IssueCode,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("JSON invalid sau câmp necunoscut: {0}")]
    Parse(String),
    #[error("configurație respinsă: {} problemă(e)", .0.len())]
    Invalid(Vec<Issue>),
}

/// Colector de probleme folosit de validatori.
#[derive(Debug, Default)]
pub(crate) struct Issues(pub(crate) Vec<Issue>);

impl Issues {
    pub(crate) fn push(&mut self, path: &str, code: IssueCode, message: impl Into<String>) {
        self.0.push(Issue {
            path: path.to_string(),
            code,
            message: message.into(),
        });
    }

    pub(crate) fn require(&mut self, condition: bool, path: &str, code: IssueCode, message: &str) {
        if !condition {
            self.push(path, code, message);
        }
    }

    pub(crate) fn finite_non_negative(&mut self, value: f64, path: &str) {
        if !value.is_finite() || value < 0.0 {
            self.push(
                path,
                IssueCode::InvalidValue,
                "suma trebuie să fie finită și ≥ 0",
            );
        }
    }

    pub(crate) fn relative_path(&mut self, value: &str, path: &str) {
        if let Err(err) = crate::paths::parse_relative(value) {
            self.push(path, IssueCode::InvalidValue, err.to_string());
        }
    }
}

/// Adaugă câte o problemă pentru fiecare valoare placeholder (`<SELECT_...>`).
pub(crate) fn collect_placeholders<T: Serialize>(value: &T, issues: &mut Issues) {
    match serde_json::to_value(value) {
        Ok(json) => walk_placeholders(&json, "$", issues),
        Err(err) => issues.push("$", IssueCode::InvalidValue, err.to_string()),
    }
}

fn walk_placeholders(value: &serde_json::Value, path: &str, issues: &mut Issues) {
    match value {
        serde_json::Value::String(s) if contains_placeholder(s) => issues.push(
            path,
            IssueCode::Placeholder,
            "valoarea este un placeholder și trebuie aleasă în wizard",
        ),
        serde_json::Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                walk_placeholders(item, &format!("{path}[{i}]"), issues);
            }
        }
        serde_json::Value::Object(map) => {
            for (key, item) in map {
                walk_placeholders(item, &format!("{path}.{key}"), issues);
            }
        }
        _ => {}
    }
}

/// Recunoaște `<NUME_IN_MAJUSCULE>` oriunde în text.
pub fn contains_placeholder(text: &str) -> bool {
    let mut rest = text;
    while let Some(start) = rest.find('<') {
        let after = &rest[start + 1..];
        if let Some(end) = after.find('>') {
            let inner = &after[..end];
            if inner.len() >= 3
                && inner
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            {
                return true;
            }
            rest = &after[end..];
        } else {
            break;
        }
    }
    false
}

/// Cale absolută pentru rădăcini selectate de utilizator (Windows sau POSIX).
/// Căile UNC sunt refuzate: starea și proiectul nu stau pe share-uri de rețea.
pub(crate) fn is_absolute_local_path(path: &str) -> bool {
    let unified = path.replace('\\', "/");
    if unified.starts_with("//") {
        return false;
    }
    let mut chars = unified.chars();
    let drive = matches!(
        (chars.next(), chars.next(), chars.next()),
        (Some(l), Some(':'), Some('/')) if l.is_ascii_alphabetic()
    );
    drive || unified.starts_with('/')
}

/// Componentele unui URL simplu `scheme://host[:port][/path]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleUrl {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
}

/// Parser minimal, intenționat restrictiv: fără userinfo, fără spații.
pub fn parse_simple_url(url: &str) -> Option<SimpleUrl> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if !matches!(scheme.as_str(), "http" | "https") || url.chars().any(char::is_whitespace) {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.is_empty() || authority.contains('@') {
        return None;
    }
    let (host, port) = if let Some(v6) = authority.strip_prefix('[') {
        let (host, tail) = v6.split_once(']')?;
        let port = match tail.strip_prefix(':') {
            Some(p) => Some(p.parse().ok()?),
            None if tail.is_empty() => None,
            None => return None,
        };
        (format!("[{}]", host.to_ascii_lowercase()), port)
    } else {
        match authority.split_once(':') {
            Some((h, p)) => (h.to_ascii_lowercase(), Some(p.parse().ok()?)),
            None => (authority.to_ascii_lowercase(), None),
        }
    };
    if host.is_empty() {
        return None;
    }
    Some(SimpleUrl { scheme, host, port })
}

pub(crate) fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "[::1]")
}

/// Hostname DNS (nu IP literal, nu `localhost`) pentru surse externe.
pub fn is_public_hostname(host: &str) -> bool {
    let labels: Vec<&str> = host.split('.').collect();
    host.len() <= 253
        && labels.len() >= 2
        && host != "localhost"
        && !host.ends_with(".localhost")
        && host.parse::<std::net::IpAddr>().is_err()
        && labels.iter().all(|l| {
            !l.is_empty()
                && l.len() <= 63
                && !l.starts_with('-')
                && !l.ends_with('-')
                && l.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        })
        && labels
            .last()
            .is_some_and(|tld| tld.chars().all(|c| c.is_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_detection() {
        assert!(contains_placeholder("<SELECT_PROJECT_DIRECTORY>"));
        assert!(contains_placeholder("C:/<SELECT_VAULT>/x"));
        assert!(!contains_placeholder("a < b > c"));
        assert!(!contains_placeholder("<b>"));
        assert!(!contains_placeholder("qwen2.5-coder:7b"));
    }

    #[test]
    fn absolute_local_paths() {
        assert!(is_absolute_local_path("D:\\Roblox\\ObbyVulcanic"));
        assert!(is_absolute_local_path("/home/user/proiect"));
        assert!(!is_absolute_local_path("\\\\server\\share"));
        assert!(!is_absolute_local_path("relative/path"));
        assert!(!is_absolute_local_path("C:relative"));
    }

    #[test]
    fn simple_url_parsing() {
        let u = parse_simple_url("http://127.0.0.1:11434").expect("url");
        assert_eq!(
            (u.scheme.as_str(), u.host.as_str(), u.port),
            ("http", "127.0.0.1", Some(11434))
        );
        let u = parse_simple_url("https://Create.Roblox.com/docs").expect("url");
        assert_eq!(u.host, "create.roblox.com");
        assert_eq!(
            parse_simple_url("http://[::1]:11434").expect("url").host,
            "[::1]"
        );
        assert!(parse_simple_url("https://user@evil.com").is_none());
        assert!(parse_simple_url("ftp://x.com").is_none());
        assert!(parse_simple_url("https://x.com:99999").is_none());
    }

    #[test]
    fn public_hostnames() {
        assert!(is_public_hostname("create.roblox.com"));
        assert!(is_public_hostname("rojo.space"));
        assert!(!is_public_hostname("localhost"));
        assert!(!is_public_hostname("127.0.0.1"));
        assert!(!is_public_hostname("169.254.169.254"));
        assert!(!is_public_hostname("intranet"));
        assert!(!is_public_hostname("Create.Roblox.com"));
    }
}
