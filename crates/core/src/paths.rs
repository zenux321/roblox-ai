//! Validarea lexicală a căilor relative la rădăcina proiectului.
//!
//! Regulile sunt deliberat stricte și aplică semantica Windows și pe alte
//! sisteme: separatori `\` și `/`, comparare fără diferențe de majuscule,
//! nume rezervate, ADS și nume scurte 8.3. Verificarea lexicală NU acoperă
//! junction/symlink sau curse între validare și scriere (S03 complet, S04):
//! acestea cer deschidere sigură pe handle, implementată în worker pe Windows.

use thiserror::Error;

const MAX_PATH_CHARS: usize = 1024;
const RESERVED_NAMES: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PathError {
    #[error("calea este goală")]
    Empty,
    #[error("calea este prea lungă")]
    TooLong,
    #[error("calea conține caractere de control")]
    ControlCharacter,
    #[error("căile de dispozitiv (\\\\?\\, \\\\.\\) nu sunt permise")]
    DevicePath,
    #[error("căile UNC nu sunt permise")]
    Unc,
    #[error("calea trebuie să fie relativă la proiect")]
    Absolute,
    #[error("`..` nu este permis")]
    ParentTraversal,
    #[error("fluxurile alternative NTFS (`:`) nu sunt permise")]
    AlternateDataStream,
    #[error("numele rezervat Windows `{0}` nu este permis")]
    ReservedName(String),
    #[error("componenta `{0}` nu este permisă")]
    InvalidComponent(String),
}

/// Cale relativă normalizată (separator `/`, fără `.` și fără componente goale).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelPath {
    components: Vec<String>,
}

impl RelPath {
    pub fn components(&self) -> &[String] {
        &self.components
    }

    pub fn as_string(&self) -> String {
        self.components.join("/")
    }

    /// Adevărat dacă această cale este `scope` sau se află sub `scope`.
    /// Compararea ignoră majusculele, ca pe NTFS.
    pub fn is_within(&self, scope: &RelPath) -> bool {
        self.components.len() >= scope.components.len()
            && self
                .components
                .iter()
                .zip(&scope.components)
                .all(|(a, b)| a.to_lowercase() == b.to_lowercase())
    }
}

impl std::fmt::Display for RelPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_string())
    }
}

/// Parsează o cale relativă la rădăcina proiectului și respinge orice formă
/// care ar putea ieși din rădăcină sau ar putea crea alias-uri pe Windows.
pub fn parse_relative(input: &str) -> Result<RelPath, PathError> {
    if input.trim().is_empty() {
        return Err(PathError::Empty);
    }
    if input.chars().count() > MAX_PATH_CHARS {
        return Err(PathError::TooLong);
    }
    if input.chars().any(char::is_control) {
        return Err(PathError::ControlCharacter);
    }
    let unified = input.replace('\\', "/");
    if unified.starts_with("//?/") || unified.starts_with("//./") {
        return Err(PathError::DevicePath);
    }
    if unified.starts_with("//") {
        return Err(PathError::Unc);
    }
    if unified.starts_with('/') || has_drive_prefix(&unified) {
        return Err(PathError::Absolute);
    }

    let mut components = Vec::new();
    for part in unified.split('/') {
        match part {
            "" | "." => continue,
            ".." => return Err(PathError::ParentTraversal),
            _ => {}
        }
        validate_component(part)?;
        components.push(part.to_string());
    }
    if components.is_empty() {
        return Err(PathError::Empty);
    }
    Ok(RelPath { components })
}

fn has_drive_prefix(path: &str) -> bool {
    let mut chars = path.chars();
    matches!((chars.next(), chars.next()), (Some(letter), Some(':')) if letter.is_ascii_alphabetic())
}

fn validate_component(part: &str) -> Result<(), PathError> {
    if part.contains(':') {
        return Err(PathError::AlternateDataStream);
    }
    if part.ends_with('.') || part.ends_with(' ') || part.starts_with(' ') {
        return Err(PathError::InvalidComponent(part.to_string()));
    }
    if part
        .chars()
        .any(|c| matches!(c, '<' | '>' | '"' | '|' | '?' | '*'))
    {
        return Err(PathError::InvalidComponent(part.to_string()));
    }
    if looks_like_short_name(part) {
        return Err(PathError::InvalidComponent(part.to_string()));
    }
    let stem = part.split('.').next().unwrap_or(part).to_lowercase();
    if RESERVED_NAMES.contains(&stem.trim_end()) {
        return Err(PathError::ReservedName(part.to_string()));
    }
    Ok(())
}

/// Numele scurte 8.3 (`PROGRA~1`) pot fi alias pentru alt director și ar ocoli scope-ul.
fn looks_like_short_name(part: &str) -> bool {
    let bytes = part.as_bytes();
    bytes
        .windows(2)
        .any(|w| w[0] == b'~' && w[1].is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(path: &str) -> String {
        parse_relative(path).expect("cale validă").as_string()
    }

    #[test]
    fn normalizes_separators_and_dots() {
        assert_eq!(
            ok("src/server/Main.server.luau"),
            "src/server/Main.server.luau"
        );
        assert_eq!(
            ok("src\\client\\HUD.client.luau"),
            "src/client/HUD.client.luau"
        );
        assert_eq!(ok("./src//a.luau"), "src/a.luau");
        assert_eq!(
            ok("assets/thumbnails/ăîșțâ.png"),
            "assets/thumbnails/ăîșțâ.png"
        );
    }

    #[test]
    fn rejects_escapes_and_aliases() {
        let cases = [
            ("", PathError::Empty),
            (".", PathError::Empty),
            ("../secret", PathError::ParentTraversal),
            ("src/../../x", PathError::ParentTraversal),
            ("src\\..\\..\\x", PathError::ParentTraversal),
            ("/etc/passwd", PathError::Absolute),
            ("\\Windows\\System32", PathError::Absolute),
            ("C:\\Users\\x", PathError::Absolute),
            ("c:relative", PathError::Absolute),
            ("\\\\server\\share\\f", PathError::Unc),
            ("//server/share/f", PathError::Unc),
            ("\\\\?\\C:\\x", PathError::DevicePath),
            ("\\\\.\\PhysicalDrive0", PathError::DevicePath),
            ("src/file.txt:hidden", PathError::AlternateDataStream),
            ("src/a\u{0}b", PathError::ControlCharacter),
            ("src/a\nb", PathError::ControlCharacter),
        ];
        for (input, expected) in cases {
            assert_eq!(parse_relative(input), Err(expected), "intrare: {input:?}");
        }
    }

    #[test]
    fn rejects_windows_specific_components() {
        assert!(matches!(
            parse_relative("src/CON"),
            Err(PathError::ReservedName(_))
        ));
        assert!(matches!(
            parse_relative("src/nul.txt"),
            Err(PathError::ReservedName(_))
        ));
        assert!(matches!(
            parse_relative("Lpt1.log"),
            Err(PathError::ReservedName(_))
        ));
        assert!(matches!(
            parse_relative("src/a."),
            Err(PathError::InvalidComponent(_))
        ));
        assert!(matches!(
            parse_relative("src/a "),
            Err(PathError::InvalidComponent(_))
        ));
        assert!(matches!(
            parse_relative("SRC~1/a"),
            Err(PathError::InvalidComponent(_))
        ));
        assert!(matches!(
            parse_relative("src/a*b"),
            Err(PathError::InvalidComponent(_))
        ));
    }

    #[test]
    fn scope_check_is_case_insensitive_and_component_based() {
        let scope = parse_relative("src").expect("scope");
        assert!(
            parse_relative("SRC/Main.luau")
                .expect("p")
                .is_within(&scope)
        );
        assert!(parse_relative("src").expect("p").is_within(&scope));
        assert!(
            !parse_relative("src2/Main.luau")
                .expect("p")
                .is_within(&scope)
        );
        assert!(!parse_relative("reports/x.md").expect("p").is_within(&scope));
    }
}
