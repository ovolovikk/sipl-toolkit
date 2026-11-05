use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct State(BTreeMap<String, i64>);

impl State {
    pub fn new() -> State {
        State::default()
    }

    pub fn get(&self, name: &str) -> Option<i64> {
        self.0.get(name).copied()
    }

    pub fn set(&mut self, name: &str, value: i64) {
        self.0.insert(name.to_string(), value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, i64)> {
        self.0.iter().map(|(name, &value)| (name.as_str(), value))
    }

    pub fn parse_binding(text: &str) -> Result<(String, i64), String> {
        let Some((name, value)) = text.split_once('=') else {
            return Err(format!("очікувалось `ЗМІННА=ЧИСЛО`, отримано `{text}`"));
        };
        let valid_name = name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !valid_name {
            return Err(format!("`{name}` не є іменем змінної"));
        }
        match value.parse::<i64>() {
            Ok(value) => Ok((name.to_string(), value)),
            Err(_) => Err(format!("`{value}` не є 64-бітним цілим числом")),
        }
    }
}

impl<const N: usize> From<[(&str, i64); N]> for State {
    fn from(bindings: [(&str, i64); N]) -> State {
        let mut state = State::new();
        for (name, value) in bindings {
            state.set(name, value);
        }
        state
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bindings: Vec<String> = self
            .iter()
            .map(|(name, value)| format!("{name} ↦ {value}"))
            .collect();
        write!(f, "[{}]", bindings.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_sorted_by_name() {
        let state = State::from([("Y", 0), ("R", 1024), ("X", 2)]);
        assert_eq!(state.to_string(), "[R ↦ 1024, X ↦ 2, Y ↦ 0]");
        assert_eq!(State::new().to_string(), "[]");
    }

    #[test]
    fn set_overwrites() {
        let mut state = State::from([("X", 1)]);
        state.set("X", 5);
        assert_eq!(state.get("X"), Some(5));
        assert_eq!(state.get("Y"), None);
    }

    #[test]
    fn parse_binding() {
        assert_eq!(State::parse_binding("X=5"), Ok(("X".to_string(), 5)));
        assert_eq!(State::parse_binding("_a1=-7"), Ok(("_a1".to_string(), -7)));
        assert!(State::parse_binding("X").is_err());
        assert!(State::parse_binding("=5").is_err());
        assert!(State::parse_binding("1X=5").is_err());
        assert!(State::parse_binding("X=five").is_err());
        assert!(State::parse_binding("X=99999999999999999999").is_err());
    }
}
