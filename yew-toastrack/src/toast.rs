//! Toast itself

/// The "level" of a toast
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Danger,
}

impl ToastLevel {
    pub fn classname(self) -> &'static str {
        match self {
            ToastLevel::Info => "is-info",
            ToastLevel::Success => "is-success",
            ToastLevel::Warning => "is-warning",
            ToastLevel::Danger => "is-danger",
        }
    }
}

/// A toast message
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toast {
    message: String,
    level: ToastLevel,
    lifetime: Option<usize>,
}

impl Toast {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            level: ToastLevel::Info,
            lifetime: None,
        }
    }

    pub fn with_level(self, level: ToastLevel) -> Self {
        Self { level, ..self }
    }

    pub fn with_lifetime(self, millis: Option<usize>) -> Self {
        Self {
            lifetime: millis,
            ..self
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn level(&self) -> ToastLevel {
        self.level
    }

    pub fn lifetime(&self) -> Option<usize> {
        self.lifetime
    }
}
