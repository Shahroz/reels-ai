//! Document update parameter transformation and business logic.
//!
//! This module handles conversion from HTTP request parameters to structured
//! database update parameters. It implements the business logic for public
//! document ownership coordination and parameter validation.

/// Structured parameters for document database updates.
///
/// This struct encapsulates all the parameters needed for document updates,
/// including the business logic for coordinating is_public and user_id fields.
/// It provides a clear separation between HTTP request handling and database operations.
#[derive(std::fmt::Debug)]
pub struct DocumentUpdateParams {
    pub title: std::option::Option<std::string::String>,
    pub content: std::option::Option<std::string::String>,
    pub is_task: std::option::Option<bool>,
    pub include_research: std::option::Option<crate::db::document_research_usage::DocumentResearchUsage>,
    pub is_public: std::option::Option<bool>,
    pub user_id: std::option::Option<uuid::Uuid>,
}

impl DocumentUpdateParams {
    /// Creates update parameters from HTTP request with business logic applied.
    ///
    /// This function implements the core business logic for document visibility:
    /// - Public documents have user_id = NULL (accessible to all users)
    /// - Private documents have user_id = authenticated_user_id (owner access)
    /// - Unchanged visibility preserves existing ownership
    pub fn from_request(req: &crate::routes::documents::update_document_request::UpdateDocumentRequest, authenticated_user_id: uuid::Uuid) -> Self {
        let (is_public_to_set, user_id_to_set) = if let std::option::Option::Some(requested_public) = req.is_public {
            // For public documents, set user_id to NULL so they're accessible to all users
            // For private documents, set user_id to the authenticated user (the one making the change)
            let new_user_id = if requested_public { std::option::Option::None } else { std::option::Option::Some(authenticated_user_id) };
            (std::option::Option::Some(requested_public), new_user_id)
        } else {
            // If is_public is not being updated, keep existing values
            (std::option::Option::None, std::option::Option::None)
        };
        
        Self {
            title: req.title.clone(),
            content: req.content.clone(),
            is_task: req.is_task,
            include_research: req.include_research.clone(),
            is_public: is_public_to_set,
            user_id: user_id_to_set,
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests for DocumentUpdateParams business logic and parameter transformation.
    
    #[test]
    fn test_from_request_with_public_document() {
        let req = crate::routes::documents::update_document_request::UpdateDocumentRequest {
            title: std::option::Option::Some("Test Title".to_string()),
            content: std::option::Option::Some("Test Content".to_string()),
            is_task: std::option::Option::Some(false),
            include_research: std::option::Option::None,
            is_public: std::option::Option::Some(true),
        };
        
        let user_id = uuid::Uuid::new_v4();
        let params = super::DocumentUpdateParams::from_request(&req, user_id);
        
        assert_eq!(params.title, req.title);
        assert_eq!(params.content, req.content);
        assert_eq!(params.is_task, req.is_task);
        assert_eq!(params.is_public, std::option::Option::Some(true));
        assert_eq!(params.user_id, std::option::Option::None); // Public documents have no owner
    }

    #[test]
    fn test_from_request_with_private_document() {
        let req = crate::routes::documents::update_document_request::UpdateDocumentRequest {
            title: std::option::Option::Some("Test Title".to_string()),
            content: std::option::Option::Some("Test Content".to_string()),
            is_task: std::option::Option::Some(false),
            include_research: std::option::Option::None,
            is_public: std::option::Option::Some(false),
        };
        
        let user_id = uuid::Uuid::new_v4();
        let params = super::DocumentUpdateParams::from_request(&req, user_id);
        
        assert_eq!(params.title, req.title);
        assert_eq!(params.content, req.content);
        assert_eq!(params.is_task, req.is_task);
        assert_eq!(params.is_public, std::option::Option::Some(false));
        assert_eq!(params.user_id, std::option::Option::Some(user_id)); // Private documents have owner
    }

    #[test]
    fn test_from_request_without_visibility_change() {
        let req = crate::routes::documents::update_document_request::UpdateDocumentRequest {
            title: std::option::Option::Some("Test Title".to_string()),
            content: std::option::Option::Some("Test Content".to_string()),
            is_task: std::option::Option::Some(false),
            include_research: std::option::Option::None,
            is_public: std::option::Option::None,
        };
        
        let user_id = uuid::Uuid::new_v4();
        let params = super::DocumentUpdateParams::from_request(&req, user_id);
        
        assert_eq!(params.title, req.title);
        assert_eq!(params.content, req.content);
        assert_eq!(params.is_task, req.is_task);
        assert_eq!(params.is_public, std::option::Option::None); // No visibility change
        assert_eq!(params.user_id, std::option::Option::None); // No ownership change
    }
} 