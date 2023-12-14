use std::sync::OnceLock;

use tokio::sync::OnceCell;

pub mod murmur;
pub mod token;

/// Check auth
pub(super) static CHECK_AUTH: OnceCell<Option<String>> = OnceCell::const_new();
/// Token secret
static TOKEN_SECRET: OnceLock<String> = OnceLock::new();
/// Token expire
pub(super) const EXP: u64 = 3600 * 24;
