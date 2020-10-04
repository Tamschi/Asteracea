use syn::{Error, Result};

pub trait MapMessage {
    fn map_message(self, message: &str) -> Self;
}

impl<T> MapMessage for Result<T> {
    fn map_message(self, message: &str) -> Self {
        self.map_err(|error| Error::new(error.span(), message))
    }
}
