pub mod agent;
pub mod check;
// Legacy CI module removed - using declarative CI detection
pub mod detectors;
pub mod engine;
pub mod schema;
pub mod traits;

pub use traits::terminal::TerminalTraits;
