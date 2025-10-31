pub mod app_setup;pub mod assets;
pub mod utils;
pub mod auth;
pub mod config;
pub mod conversation;
pub mod handlers;
pub mod session;
pub mod state;
pub mod tools;
pub mod types;
pub mod websocket;
pub mod evaluator;
pub mod setup; // Added setup module
pub mod lib_runner;

pub mod openapi;
pub use setup::setup_agentloop_core; // Re-export the public setup function
pub use lib_runner::run_research_task; // Re-export the new public function
