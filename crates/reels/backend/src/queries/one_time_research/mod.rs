//! Module for all database queries related to one-time research tasks.
//!
//! This module follows the one-item-per-file pattern, where each file
//! contains a single query function. The functions are re-exported here
//! for convenient access from other parts of the application.

pub mod create_one_time_research;
pub mod delete_one_time_research;
pub mod get_one_time_research_by_id;
pub mod get_one_time_research_by_id_internal;
pub mod list_one_time_researches_by_user_id;
pub mod update_one_time_research_on_finish;
pub mod update_one_time_research_on_start;
pub mod update_one_time_research_status;
pub mod update_one_time_research_progress;

pub use create_one_time_research::create_one_time_research;
pub use delete_one_time_research::delete_one_time_research;
pub use get_one_time_research_by_id::get_one_time_research_by_id;
pub use get_one_time_research_by_id_internal::get_one_time_research_by_id_internal;
pub use list_one_time_researches_by_user_id::list_one_time_researches_by_user_id;
pub use update_one_time_research_on_finish::update_one_time_research_on_finish;
pub use update_one_time_research_on_start::update_one_time_research_on_start;
pub use update_one_time_research_status::update_one_time_research_status;
pub use update_one_time_research_progress::update_one_time_research_progress;