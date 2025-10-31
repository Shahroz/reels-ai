//! Creates a new style record with complete access and creator details.
//!
//! This function creates a new style with the provided details and returns
//! a complete StyleResponse including creator email and current user access level.
//! Supports both public and private styles with proper access level calculation.

/// Parameters for creating a style with details
#[derive(Debug)]
pub struct CreateStyleWithDetailsParams {
    pub style_id: uuid::Uuid,
    pub user_id: std::option::Option<uuid::Uuid>, // None for public styles
    pub name: std::string::String,
    pub html_url: std::string::String,
    pub screenshot_url: std::string::String,
    pub is_public: bool,
    pub requesting_user_id: uuid::Uuid, // For access level calculation
}

/// Creates a new style and returns complete response with creator details
/// 
/// Inserts the style record and returns full StyleResponse including creator email
/// and access level information. Handles both public and private styles properly.
#[tracing::instrument(skip(pool))]
pub async fn create_style_with_details(
    pool: &sqlx::PgPool,
    params: CreateStyleWithDetailsParams,
) -> std::result::Result<crate::routes::styles::responses::StyleResponse, sqlx::Error> {
    #[derive(sqlx::FromRow, Debug)]
    struct CreatedStyleDetails {
        id: uuid::Uuid,
        user_id: std::option::Option<uuid::Uuid>,
        name: std::string::String,
        html_url: std::string::String,
        screenshot_url: std::string::String,
        is_public: bool,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
        creator_email: std::option::Option<std::string::String>,
        current_user_access_level: std::option::Option<std::string::String>,
    }

    let result = sqlx::query_as!(
        CreatedStyleDetails,
        r#"
        WITH inserted_style AS (
            INSERT INTO styles (id, user_id, name, html_url, screenshot_url, is_public)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, name, html_url, screenshot_url, is_public, created_at, updated_at
        )
        SELECT 
            i_s.id as "id!", 
            i_s.user_id, 
            i_s.name as "name!", 
            i_s.html_url as "html_url!", 
            i_s.screenshot_url as "screenshot_url!", 
            i_s.is_public as "is_public!",
            i_s.created_at as "created_at!", 
            i_s.updated_at as "updated_at!", 
            u.email as "creator_email?",
            CASE 
                WHEN i_s.user_id = $7 THEN 'owner'::text
                ELSE NULL::text
            END AS "current_user_access_level?"
        FROM inserted_style i_s
        LEFT JOIN users u ON i_s.user_id = u.id
        "#,
        params.style_id,
        params.user_id,
        params.name,
        params.html_url,
        params.screenshot_url,
        params.is_public,
        params.requesting_user_id
    )
    .fetch_one(pool)
    .await?;

    std::result::Result::Ok(crate::routes::styles::responses::StyleResponse {
        style: crate::db::styles::Style {
            id: result.id,
            user_id: result.user_id,
            name: result.name,
            html_url: result.html_url,
            screenshot_url: result.screenshot_url,
            is_public: result.is_public,
            created_at: result.created_at,
            updated_at: result.updated_at,
        },
        creator_email: result.creator_email,
        current_user_access_level: result.current_user_access_level,
    })
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_public_style_params() {
        // Test parameter construction for public style
        let params = super::CreateStyleWithDetailsParams {
            style_id: uuid::Uuid::new_v4(),
            user_id: None, // Public style
            name: std::string::String::from("Test Public Style"),
            html_url: std::string::String::from("https://example.com/style.html"),
            screenshot_url: std::string::String::from("https://example.com/screenshot.png"),
            is_public: true,
            requesting_user_id: uuid::Uuid::new_v4(),
        };
        
        assert!(params.is_public);
        assert!(params.user_id.is_none());
        assert_eq!(params.name, "Test Public Style");
    }

    #[test]
    fn test_private_style_params() {
        // Test parameter construction for private style
        let user_id = uuid::Uuid::new_v4();
        let params = super::CreateStyleWithDetailsParams {
            style_id: uuid::Uuid::new_v4(),
            user_id: Some(user_id), // Private style
            name: std::string::String::from("Test Private Style"),
            html_url: std::string::String::from("https://example.com/style.html"),
            screenshot_url: std::string::String::from("https://example.com/screenshot.png"),
            is_public: false,
            requesting_user_id: user_id, // Same user
        };
        
        assert!(!params.is_public);
        assert!(params.user_id.is_some());
        assert_eq!(params.user_id, Some(user_id));
    }
} 