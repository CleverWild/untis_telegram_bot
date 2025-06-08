use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
enum LabeledString {
    Normal(String),
    Debug(String),
}

impl Display for LabeledString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabeledString::Normal(text) | LabeledString::Debug(text) => write!(f, "{text}"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LabeledStrings(Vec<LabeledString>);
impl LabeledStrings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_normal(&mut self, text: String) {
        self.0.push(LabeledString::Normal(text));
    }

    pub fn push_debug(&mut self, text: String) {
        self.0.push(LabeledString::Debug(text));
    }

    pub fn display_normal(&self) -> String {
        self.0
            .iter()
            .filter_map(|s| match s {
                LabeledString::Normal(text) => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn display_all(&self) -> String {
        self.to_string()
    }
}

impl Display for LabeledStrings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for text in &self.0 {
            write!(f, "{text}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_labeled_string_display() {
        let normal = LabeledString::Normal("Hello".to_string());
        let debug = LabeledString::Debug("Debug info".to_string());

        assert_eq!(format!("{normal}"), "Hello");
        assert_eq!(format!("{debug}"), "Debug info");
    }

    #[test]
    fn test_mixed_push() {
        let mut strings = LabeledStrings::new();
        strings.push_normal("Normal".to_string());
        strings.push_debug("Debug".to_string());
        strings.push_normal("Another normal".to_string());

        assert_eq!(strings.0.len(), 3);
        assert_eq!(strings.0[0], LabeledString::Normal("Normal".to_string()));
        assert_eq!(strings.0[1], LabeledString::Debug("Debug".to_string()));
        assert_eq!(
            strings.0[2],
            LabeledString::Normal("Another normal".to_string())
        );
    }

    #[test]
    fn test_display_normal() {
        let mut strings = LabeledStrings::new();
        strings.push_normal("First normal".to_string());
        strings.push_debug("Debug info".to_string());
        strings.push_normal("Second normal".to_string());
        strings.push_debug("More debug".to_string());

        let result = strings.display_normal();
        assert_eq!(result, "First normalSecond normal");
    }

    #[test]
    fn test_display_normal_empty() {
        let strings = LabeledStrings::new();
        assert_eq!(strings.display_normal(), "");
    }

    #[test]
    fn test_display_normal_only_debug() {
        let mut strings = LabeledStrings::new();
        strings.push_debug("Debug 1".to_string());
        strings.push_debug("Debug 2".to_string());

        assert_eq!(strings.display_normal(), "");
    }

    #[test]
    fn test_display_all() {
        let mut strings = LabeledStrings::new();
        strings.push_normal("Normal".to_string());
        strings.push_debug("Debug".to_string());

        let all = strings.display_all();
        let to_string = strings.to_string();

        assert_eq!(all, to_string);
        assert_eq!(all, "NormalDebug");
    }

    #[test]
    fn test_labeled_strings_display() {
        let mut strings = LabeledStrings::new();
        strings.push_normal("Hello ".to_string());
        strings.push_debug("world".to_string());
        strings.push_normal("!".to_string());

        assert_eq!(format!("{strings}"), "Hello world!");
    }

    #[test]
    fn test_empty_strings_display() {
        let strings = LabeledStrings::new();
        assert_eq!(format!("{strings}"), "");
        assert_eq!(strings.display_all(), "");
        assert_eq!(strings.display_normal(), "");
    }

    #[test]
    fn test_real_world_scenario() {
        let mut messages = LabeledStrings::new();

        // Сценарий как в реальном использовании
        messages.push_normal("Changes in timetable:\n".to_string());
        messages.push_normal("\nDate: 2024-01-15 Monday\n".to_string());
        messages.push_normal("Subject: Math\nRoom: 101 → 102\n".to_string());
        messages.push_debug("Debug: Lesson ID 12345\n".to_string());

        let normal_output = messages.display_normal();
        let all_output = messages.display_all();

        assert!(normal_output.contains("Changes in timetable"));
        assert!(normal_output.contains("Math"));
        assert!(normal_output.contains("101 → 102"));
        assert!(!normal_output.contains("Debug"));

        assert!(all_output.contains("Changes in timetable"));
        assert!(all_output.contains("Math"));
        assert!(all_output.contains("Debug: Lesson ID 12345"));
    }

    #[test]
    fn test_debug_only_scenario() {
        let mut messages = LabeledStrings::new();
        messages.push_debug("Debug info 1".to_string());
        messages.push_debug("Debug info 2".to_string());

        assert_eq!(messages.display_normal(), "");
        assert_eq!(messages.display_all(), "Debug info 1Debug info 2");
    }
}
