//! Session monitoring module
//! Provides visibility into sub-agent session states

pub mod state;
pub mod monitor_trait;
pub mod opencode;
pub mod claude;

pub use state::{AgentState, SessionActivity};
pub use monitor_trait::SessionMonitor;
pub use opencode::OpenCodeMonitor;
pub use claude::ClaudeMonitor;
