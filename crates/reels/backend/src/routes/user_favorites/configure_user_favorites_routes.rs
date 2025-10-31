//! Configures the routes for the user favorites API.
use actix_web::web;
use crate::routes::user_favorites::create_favorite::create_favorite;
use crate::routes::user_favorites::list_favorites::list_favorites;
use crate::routes::user_favorites::delete_favorite::delete_favorite;
use crate::routes::user_favorites::toggle_favorite::toggle_favorite;
use crate::routes::user_favorites::add_favorite_prompt::add_favorite_prompt;
use crate::routes::user_favorites::list_favorite_prompts::list_favorite_prompts;
use crate::routes::user_favorites::remove_favorite_prompt::remove_favorite_prompt;

/// Mounts the user favorites-related routes to the Actix web application.
///
/// # Arguments
///
/// * `cfg` - A mutable reference to the Actix web `ServiceConfig`.
pub fn configure_user_favorites_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(create_favorite)      // POST /
            .service(list_favorites)       // GET /
            .service(delete_favorite)      // DELETE /{favorite_id}
            .service(toggle_favorite)      // POST /toggle
            .service(add_favorite_prompt)  // POST /prompts
            .service(list_favorite_prompts) // GET /prompts
            .service(remove_favorite_prompt) // DELETE /prompts/{prompt_id}
    );
} 