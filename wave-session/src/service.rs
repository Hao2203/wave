use crate::user::{self, CurrentUser, User};

pub struct Service<'node> {
    node: &'node iroh::node::FsNode,
    user: User,
}

impl<'node> Service<'node> {
    pub async fn new(node: &'node iroh::node::FsNode, user: Option<&impl CurrentUser>) -> Self {
        let user = if let Some(user) = user {
            user::User::from_current_user(user)
        } else {
            let author = node.authors().default().await.unwrap();
            let name = author.to_string();
            user::User::new(name, author)
        };
        Self { node, user }
    }

    pub async fn list_sessions(&self) {}
}
