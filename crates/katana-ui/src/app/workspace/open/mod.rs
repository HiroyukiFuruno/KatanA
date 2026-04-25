/* WHY: Refactored workspace open entry point to maintain a clean structure and manage architectural complexity via specialized sub-modules. */

mod handlers;
mod registration;
mod session;

pub(crate) use handlers::WorkspaceOpenHandlersOps;
