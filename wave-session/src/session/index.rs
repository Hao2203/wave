use crate::{
    error::{ErrorKind, Result},
    SessionHandle,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionIndex {
    name: String,
}

impl SessionIndex {
    pub fn new(name: impl AsRef<str>) -> Result<Self> {
        if name.as_ref().len() > 64 {
            return Err(ErrorKind::SessionNameTooLong)?;
        }
        Ok(Self {
            name: name.as_ref().to_owned(),
        })
    }
}

impl SessionHandle for SessionIndex {
    fn name(&self) -> &str {
        &self.name
    }
}
