use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum LoadError {
    FileNotExist(String),
    PermissionDenied(String),
}

impl Display for LoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::FileNotExist(string) | LoadError::PermissionDenied(string) => {
                write!(f, "{string}")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SaveError {
    error: String,
}

impl SaveError {
    pub fn new<T: ToString>(error: T) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}

impl Display for SaveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

#[derive(Debug, Clone)]
pub struct DeleteError {
    error: String,
}

impl DeleteError {
    pub fn new<T: ToString>(error: T) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}

impl Display for DeleteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
}
