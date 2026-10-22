//! Furnizorul de chat demonstrativ: aceleași reguli ca `src/mock/replies.ts`.
//! Nu folosește un model real și nu are acces la rețea.

use crate::dto::AssistantExtra;

pub const DEMO_MODEL_LABEL: &str = "model simulat";
pub const OWNER_NAME: &str = "Andrei";

#[derive(Debug, Clone, PartialEq)]
pub struct DemoReply {
    pub paragraphs: Vec<String>,
    pub list: Option<Vec<String>>,
    pub tag: Option<String>,
    pub extras: Vec<AssistantExtra>,
}

impl DemoReply {
    pub fn plain_text(&self) -> String {
        let mut parts = self.paragraphs.clone();
        parts.extend(self.list.clone().unwrap_or_default());
        parts.join("\n")
    }
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| text.contains(n))
}

pub fn demo_reply_for(prompt: &str) -> DemoReply {
    let text = prompt.to_lowercase();
    let simple = |p: &str, extras: Vec<AssistantExtra>| DemoReply {
        paragraphs: vec![p.into()],
        list: None,
        tag: None,
        extras,
    };
    if contains_any(&text, &["genereaz", "thumbnail", "imagin"]) {
        return simple(
            "Pentru imagini am nevoie de OpenAI Images. Îți arăt costul și cine plătește înainte de orice generare.",
            vec![AssistantExtra::ImageGenerationUnconfigured],
        );
    }
    if contains_any(
        &text,
        &["adaug", "schimb", "implement", "modific", "fă ", "fa "],
    ) {
        return simple(
            &format!(
                "Asta ar modifica jocul, așa că nu o fac direct din conversație. Pregătesc un plan pe care îl poate aproba {OWNER_NAME}."
            ),
            vec![AssistantExtra::PreparePlan],
        );
    }
    if text.contains("diferen") {
        let mut reply = simple(
            "Am comparat jocul din Studio cu fișierele și cu notițele. Platforma6 lipsește din fișiere, iar ultimul salt are 12 studs, peste limita de 10 din notă. Vrei să păstrăm varianta din Studio sau pe cea din notă?",
            Vec::new(),
        );
        reply.tag = Some("Exemplu demonstrativ".into());
        return reply;
    }
    if text.contains("checkpoint") {
        return DemoReply {
            paragraphs: vec![
                "Pe scurt: clientul rulează pe calculatorul jucătorului și poate fi modificat. Serverul e sursa de adevăr.".into(),
            ],
            list: Some(vec![
                "Clientul doar atinge piesa; evenimentul ajunge la server.".into(),
                "Serverul verifică dacă piesa e un checkpoint real.".into(),
                "Progresul se scrie numai pe server.".into(),
            ]),
            tag: Some("Doar explicație · jocul nu se modifică".into()),
            extras: Vec::new(),
        };
    }
    simple(
        "Acesta este un răspuns demonstrativ. În aplicația reală răspunde modelul local ales, folosind numai contextul bifat din vault. Nimic din joc nu se schimbă din această conversație.",
        Vec::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_match_the_typescript_mock() {
        assert_eq!(
            demo_reply_for("Generează un thumbnail").extras,
            vec![AssistantExtra::ImageGenerationUnconfigured]
        );
        assert_eq!(
            demo_reply_for("Adaugă un cronometru").extras,
            vec![AssistantExtra::PreparePlan]
        );
        assert!(demo_reply_for("Explică-mi checkpointul").list.is_some());
        assert!(demo_reply_for("salut").paragraphs[0].contains("demonstrativ"));
    }
}
