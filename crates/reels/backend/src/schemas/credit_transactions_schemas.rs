//! Credit transactions schema definitions
//!
//! This module provides schema definitions for credit transaction operations.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use bigdecimal::BigDecimal;

/// Parameters for creating a credit transaction record
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCreditTransactionParams {
    /// User ID who owns the credits
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub user_id: Uuid,
    
    /// Organization ID (optional)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub organization_id: Option<Uuid>,
    
    /// Amount of credits changed (positive for addition, negative for deduction)
    #[schema(example = "5.50", value_type = String)]
    pub credits_changed: BigDecimal,
    
    /// Previous credit balance before the transaction
    #[schema(example = "100.25", value_type = String)]
    pub previous_balance: BigDecimal,
    
    /// New credit balance after the transaction
    #[schema(example = "94.75", value_type = String)]
    pub new_balance: BigDecimal,
    
    /// Source of the action (e.g., "api", "stripe_webhook_event")
    #[schema(example = "api")]
    pub action_source: String,
    
    /// Type of action performed (e.g., "retouch_image", "buy_credits")
    #[schema(example = "retouch_image")]
    pub action_type: String,
    
    /// Entity ID related to the transaction (optional)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub entity_id: Option<Uuid>,
}
