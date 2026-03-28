pub mod auth;
pub mod proxy;

pub use auth::CoreAuth;
pub use proxy::{chat_completions, completions, embeddings};
