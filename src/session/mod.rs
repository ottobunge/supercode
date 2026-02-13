//! Session management module

pub mod manager;
pub mod provider;
pub mod opencode;
pub mod opencode_provider;
pub mod claude;
pub mod claude_provider;

pub use manager::SessionManager;
pub use provider::{SessionHandle, SessionProvider, SessionStatus};
pub use opencode::OpenCodeClient;
pub use opencode_provider::OpenCodeProvider;
pub use claude::ClaudeClient;
pub use claude_provider::ClaudeProvider;
