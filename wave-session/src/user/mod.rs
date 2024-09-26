use iroh::docs::AuthorId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct User {
    name: String,
    author_id: AuthorId,
}

impl User {
    pub fn new(name: impl AsRef<str>, author_id: AuthorId) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            author_id,
        }
    }

    pub fn from_current_user(user: &impl CurrentUser) -> Self {
        Self::new(user.name(), user.author_id())
    }
}

pub trait CurrentUser {
    fn name(&self) -> &str;

    fn author_id(&self) -> AuthorId;
}

impl CurrentUser for User {
    fn name(&self) -> &str {
        &self.name
    }
    fn author_id(&self) -> AuthorId {
        self.author_id
    }
}
