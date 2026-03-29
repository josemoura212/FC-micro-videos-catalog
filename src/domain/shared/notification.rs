use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Notification {
    errors: HashMap<String, Vec<String>>,
}

impl Notification {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, error: &str, field: Option<&str>) {
        let key = field.unwrap_or("_global").to_string();
        self.errors.entry(key).or_default().push(error.to_string());
    }

    pub fn set_error(&mut self, errors: Vec<String>, field: Option<&str>) {
        let key = field.unwrap_or("_global").to_string();
        self.errors.insert(key, errors);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub const fn errors(&self) -> &HashMap<String, Vec<String>> {
        &self.errors
    }

    pub fn copy_errors(&mut self, other: &Self) {
        for (key, values) in &other.errors {
            self.errors
                .entry(key.clone())
                .or_default()
                .extend(values.clone());
        }
    }

    pub fn to_error_messages(&self) -> Vec<String> {
        self.errors
            .iter()
            .flat_map(|(field, messages)| {
                messages
                    .iter()
                    .map(move |msg| format!("{field}: {msg}"))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_empty_notification() {
        let notification = Notification::new();
        assert!(!notification.has_errors());
    }

    #[test]
    fn should_add_error_with_field() {
        let mut notification = Notification::new();
        notification.add_error("name is required", Some("name"));
        assert!(notification.has_errors());
        assert_eq!(
            notification.errors().get("name").unwrap(),
            &vec!["name is required".to_string()]
        );
    }

    #[test]
    fn should_add_global_error() {
        let mut notification = Notification::new();
        notification.add_error("something went wrong", None);
        assert!(notification.has_errors());
        assert!(notification.errors().contains_key("_global"));
    }

    #[test]
    fn should_copy_errors_from_another_notification() {
        let mut n1 = Notification::new();
        n1.add_error("error1", Some("field1"));

        let mut n2 = Notification::new();
        n2.add_error("error2", Some("field2"));

        n1.copy_errors(&n2);
        assert!(n1.errors().contains_key("field1"));
        assert!(n1.errors().contains_key("field2"));
    }
}
