//! Content generation functions for the Content Studio.
//!
//! This module provides standalone functions for generating and transforming content
//! using LLMs, following the same pattern as creative_generation_service.rs.

use crate::db::documents::Document;
use crate::queries::documents::insert_document_entry::{insert_document_entry, InsertedDocumentData};
use llm::llm_typed_unified::vendor_model::VendorModel;
use llm::vendors::gemini::gemini_model::GeminiModel;
use llm::llm_typed_unified::llm::llm;
use sqlx::PgPool;
use uuid::Uuid;
use log;

/// Transform an existing document using LLM with a transformation prompt
/// 
/// Similar to process_single_creative_format_for_generation but for document content
pub async fn transform_document_content(
    pool: &PgPool,
    source_document: &Document,
    transformation_prompt: &str,
    user_id: Uuid,
    target_title: Option<String>,
) -> Result<Document, String> {
    // Use hardcoded Gemini 2.5 Pro model (no env variables)
    let models = vec![VendorModel::Gemini(GeminiModel::Gemini25Pro)];
    let max_attempts = 3;

    let prompt = format!(
        r#"Transform the following document content based on the user's request.

ORIGINAL DOCUMENT:
Title: {}
Content: {}

TRANSFORMATION REQUEST: {}

TASK: Please provide a transformed version of the document content that addresses the user's request while maintaining the document's core purpose and structure where appropriate. 

Output only the transformed content text without any explanations, markdown formatting, or additional commentary."#,
        &source_document.title,
        &source_document.content,
        transformation_prompt
    );

    log::info!("Transforming document {} with prompt length: {}", source_document.id, prompt.len());

    let mut attempt = 0;
    loop {
        attempt += 1;
        if models.is_empty() {
            return Err("No LLM models available.".to_string());
        }
        
        let model_idx = (attempt - 1) % models.len();
        let model_to_use = models[model_idx].clone();

        log::info!(
            "Document transformation attempt {}/{} using model {:?}",
            attempt,
            max_attempts,
            model_to_use
        );

        let llm_result = llm(
            false, // debug_mode
            &prompt,
            vec![model_to_use],
            1, // internal retries
        ).await;

        match llm_result {
            Ok(generated_content) => {
                let trimmed_content = generated_content.trim().to_string();
                
                // Basic validation - ensure we got reasonable content
                if trimmed_content.len() >= 10 {
                    // Create new document with transformed content
                    let new_title = target_title.clone().unwrap_or_else(|| {
                        format!("Transformed: {}", &source_document.title)
                    });

                    // Create transaction for document insertion
                    let mut tx = pool.begin().await.map_err(|e| format!("Failed to begin transaction: {}", e))?;
                    
                    let new_document_result = insert_document_entry(
                        &mut tx,
                        Some(user_id), // user_id as Option<Uuid>
                        &new_title,
                        &trimmed_content,
                        &[], // sources (empty for generated content)
                        false, // is_public
                        false, // is_task
                        None,  // include_research
                        source_document.collection_id, // Same collection as source
                    ).await;

                    match new_document_result {
                        Ok(inserted_data) => {
                            // Commit the transaction
                            tx.commit().await.map_err(|e| format!("Failed to commit transaction: {}", e))?;
                            
                            // Convert InsertedDocumentData to Document
                            let document = Document {
                                id: inserted_data.id,
                                user_id: Some(user_id),
                                title: new_title.clone(),
                                content: trimmed_content.clone(),
                                sources: vec![], // Empty for generated content
                                status: "Completed".to_string(),
                                created_at: inserted_data.created_at,
                                updated_at: inserted_data.updated_at,
                                is_public: false,
                                is_task: false,
                                include_research: None,
                                collection_id: source_document.collection_id,
                            };
                            
                            log::info!("Successfully transformed document {} -> {}", source_document.id, document.id);
                            return Ok(document);
                        }
                        Err(e) => {
                            // Rollback transaction on error
                            let _ = tx.rollback().await;
                            log::error!("Failed to save transformed document: {:?}", e);
                            if attempt >= max_attempts {
                                return Err(format!("Failed to save transformed document after {} attempts: {}", max_attempts, e));
                            }
                        }
                    }
                } else {
                    log::warn!(
                        "LLM output validation failed for document transformation on attempt {}/{}. Content too short: {}",
                        attempt,
                        max_attempts,
                        trimmed_content.len()
                    );
                    if attempt >= max_attempts {
                        return Err(format!("LLM output validation failed after {} attempts", max_attempts));
                    }
                }
            }
            Err(e) => {
                log::error!(
                    "LLM generation failed for document transformation on attempt {}/{}: {:?}",
                    attempt,
                    max_attempts,
                    e
                );
                if attempt >= max_attempts {
                    return Err(format!("LLM generation failed after {} attempts: {}", max_attempts, e));
                }
            }
        }

        // Small delay before retrying
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

/// Generate new content from a prompt
pub async fn generate_new_content(
    pool: &PgPool,
    prompt: &str,
    user_id: Uuid,
    title: String,
    collection_id: Option<Uuid>,
) -> Result<Document, String> {
    // Use hardcoded Gemini 2.5 Pro model (no env variables)
    let models = vec![VendorModel::Gemini(GeminiModel::Gemini25Pro)];
    let max_attempts = 3;

    let llm_prompt = format!(
        r#"Generate content based on the following request:

REQUEST: {}

TASK: Create comprehensive, well-structured content that fulfills the user's request. The content should be informative, well-organized, and appropriate for the given topic.

Output only the generated content without any explanations, markdown formatting, or additional commentary."#,
        prompt
    );

    log::info!("Generating new content with prompt length: {}", llm_prompt.len());

    let mut attempt = 0;
    loop {
        attempt += 1;
        if models.is_empty() {
            return Err("No LLM models available.".to_string());
        }
        
        let model_idx = (attempt - 1) % models.len();
        let model_to_use = models[model_idx].clone();

        log::info!(
            "Content generation attempt {}/{} using model {:?}",
            attempt,
            max_attempts,
            model_to_use
        );

        let llm_result = llm(
            false, // debug_mode
            &llm_prompt,
            vec![model_to_use],
            1, // internal retries
        ).await;

        match llm_result {
            Ok(generated_content) => {
                let trimmed_content = generated_content.trim().to_string();
                
                // Basic validation - ensure we got reasonable content
                if trimmed_content.len() >= 50 {
                    // Create new document with generated content
                    // Create transaction for document insertion
                    let mut tx = pool.begin().await.map_err(|e| format!("Failed to begin transaction: {}", e))?;
                    
                    let new_document_result = insert_document_entry(
                        &mut tx,
                        Some(user_id), // user_id as Option<Uuid>
                        &title,
                        &trimmed_content,
                        &[], // sources (empty for generated content)
                        false, // is_public
                        false, // is_task
                        None,  // include_research
                        collection_id,
                    ).await;

                    match new_document_result {
                        Ok(inserted_data) => {
                            // Commit the transaction
                            tx.commit().await.map_err(|e| format!("Failed to commit transaction: {}", e))?;
                            
                            // Convert InsertedDocumentData to Document
                            let document = Document {
                                id: inserted_data.id,
                                user_id: Some(user_id),
                                title: title.clone(),
                                content: trimmed_content.clone(),
                                sources: vec![], // Empty for generated content
                                status: "Completed".to_string(),
                                created_at: inserted_data.created_at,
                                updated_at: inserted_data.updated_at,
                                is_public: false,
                                is_task: false,
                                include_research: None,
                                collection_id,
                            };
                            
                            log::info!("Successfully generated new document {}", document.id);
                            return Ok(document);
                        }
                        Err(e) => {
                            // Rollback transaction on error
                            let _ = tx.rollback().await;
                            log::error!("Failed to save generated document: {:?}", e);
                            if attempt >= max_attempts {
                                return Err(format!("Failed to save generated document after {} attempts: {}", max_attempts, e));
                            }
                        }
                    }
                } else {
                    log::warn!(
                        "LLM output validation failed for content generation on attempt {}/{}. Content too short: {}",
                        attempt,
                        max_attempts,
                        trimmed_content.len()
                    );
                    if attempt >= max_attempts {
                        return Err(format!("LLM output validation failed after {} attempts", max_attempts));
                    }
                }
            }
            Err(e) => {
                log::error!(
                    "LLM generation failed for content generation on attempt {}/{}: {:?}",
                    attempt,
                    max_attempts,
                    e
                );
                if attempt >= max_attempts {
                    return Err(format!("LLM generation failed after {} attempts: {}", max_attempts, e));
                }
            }
        }

        // Small delay before retrying
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert!(true, "Placeholder test for content generation functions");
        // Comprehensive tests would require mocking the database and LLM calls
    }
}
