//! Response struct for successful login.
//!
//! Contains the JWT token and public user information.
use serde::Serialize;
use utoipa::ToSchema;
use crate::db::users::PublicUser;

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: std::string::String,
    #[schema(value_type = PublicUser)]
    pub user: crate::db::users::PublicUser,
}