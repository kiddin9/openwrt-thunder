use tokio::sync::OnceCell;

pub mod murmur;
pub mod token;

/// Check auth
pub(super) static CHECK_AUTH: OnceCell<Option<String>> = OnceCell::const_new();
