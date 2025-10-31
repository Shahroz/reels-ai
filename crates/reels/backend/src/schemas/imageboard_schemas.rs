use utoipa::ToSchema;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

// Common error response for imageboard webhook
#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookErrorResponse {
    pub error: String,
    pub code: String,
    pub details: Option<String>,
}

// organization.balance.fetch response
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationCreditsAllocationResponse {
    pub organization_id: Uuid,
    pub organization_name: String,
    pub credits_remaining: String,
    pub amount_cents: i32,
    pub balance_cents: i32,
    pub last_reset_date: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// organization.balance.fetch response
#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationCreditsAllocationLiteResponse {
    pub balance_cents: i32
}

// credit.deduct_transaction request
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeductCreditsRequest {
    #[schema(format = "uuid", value_type = String)]
    pub owner_id: Uuid,
    pub amount_cents: i32,
    #[schema(format = "uuid", value_type = Option<String>)]
    pub entity_id: Option<Uuid>,
}


// credit.deduct_transaction response
#[derive(Debug, Serialize, ToSchema)]
pub struct DeductCreditsResponse {
    pub message: String,
    pub credits_deducted: String,
    pub credits_remaining: String,
    pub amount_cents_remaining: i32,
    pub deducted_at: chrono::DateTime<chrono::Utc>,
}

// Bulk transaction input item (from imageboard)
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BillingTransactionRecord {
    pub id: String,
    pub user_id: String,
    pub board_id: Option<String>,
    pub amount_cents: i32,
    pub transaction_type: String,
    pub description: String,
    pub event_id: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// Bulk deduct failed item
#[derive(Debug, Serialize, ToSchema)]
pub struct FailedTransaction {
    pub reason: String,
    pub code: String,
    pub transaction_id: Option<String>,
    pub organization_id: Option<String>,
}

// credit.bulk_deduct_transaction response
#[derive(Debug, Serialize, ToSchema)]
pub struct BulkDeductCreditsResponse {
    pub succeeded_transaction_ids: Vec<String>,
    pub total_processed: usize,
    pub total_failed: usize,
    pub failed: Vec<FailedTransaction>,
    pub message: String,
}

// credit.bulk_deduct_transaction response
#[derive(Debug, Serialize, ToSchema)]
pub struct BulkDeductCreditsLiteResponse {
    pub received_count: i32,
    pub new_balance_cents: i32,
}