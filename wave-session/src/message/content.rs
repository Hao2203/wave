use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    text: String,
}

impl Content {
    pub fn new(text: String) -> Result<Self> {
        if text.len() > 1024 {
            return Err(Error::TextLengthBiggerThan1024);
        }
        Ok(Self { text })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn into_text(self) -> String {
        self.text
    }
}
