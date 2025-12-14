pub mod app;
pub mod config;
pub mod lapin;
pub mod rabbit;

// Re-export commonly used items for tests
pub use config::Config;
