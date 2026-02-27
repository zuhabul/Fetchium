//! Collaborative research workspaces — branching, merging, sync (PRD §37).

pub mod branch;
pub mod merge;
pub mod sync;
pub mod workspace;

pub use branch::{fork_session, SessionMeta};
pub use merge::{merge_sessions, Finding, MergeResult};
pub use sync::{sync_workspace, SyncReport};
pub use workspace::{SyncMethod, Workspace};
