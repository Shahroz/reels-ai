use uuid::Uuid;
use sqlx::PgPool;
use bigdecimal::BigDecimal;

use crate::app_constants::credits_constants::CREDITS_TO_CENTS_RATIO;
use crate::schemas::imageboard_schemas::{WebhookErrorResponse, BulkDeductCreditsLiteResponse, BillingTransactionRecord};

pub async fn handle_bulk_deduct_lite(
    pool: &PgPool,
    collection_id: Uuid,
    organization_id: Uuid,
    transactions: &[BillingTransactionRecord],
) -> Result<BulkDeductCreditsLiteResponse, WebhookErrorResponse> {

    // Get initial organization credit allocation
    let org_alloc = crate::queries::organization_credit_allocation::get_organization_credit_allocation_by_org_id::get_organization_credit_allocation_by_org_id(pool, organization_id)
        .await
        .map_err(|_| WebhookErrorResponse{
            error: "Internal server error".to_string(),
            code: "INTERNAL_SERVER_ERROR".to_string(),
            details: Some("Failed to retrieve organization credit allocation".to_string()),
        })?
        .ok_or_else(|| WebhookErrorResponse{
            error: "Organization credit allocation not found".to_string(),
            code: "ORG_CREDIT_ALLOCATION_NOT_FOUND".to_string(),
            details: None,
        })?;

    // Filter transactions by board_id matching collection_id and debit type only
    // All transactions belong to the same collection, so we just filter by board_id == collection_id
    let org_txs: Vec<&BillingTransactionRecord> = transactions
        .iter()
        .filter(|t| {
            // Only process debit transactions
            if t.transaction_type.to_lowercase() != "debit" {
                return false;
            }

            // Filter by board_id matching collection_id
            if let Some(board_id_str) = &t.board_id {
                match Uuid::parse_str(board_id_str) {
                    Ok(board_id) => board_id == collection_id,
                    Err(_) => false, // Invalid board_id, skip
                }
            } else {
                false // No board_id, skip
            }
        })
        .collect();

    if org_txs.is_empty() {
        // No transactions to process
        let current_balance_cents = &org_alloc.credits_remaining * BigDecimal::from(CREDITS_TO_CENTS_RATIO);
        let new_balance_cents = current_balance_cents.to_string().parse::<i32>().unwrap_or(0);
        return Ok(BulkDeductCreditsLiteResponse{
            received_count: 0,
            new_balance_cents,
        });
    }

    // Check for already processed transactions
    let ids: Vec<Uuid> = org_txs.iter().filter_map(|t| Uuid::parse_str(&t.id).ok()).collect();
    let already = crate::queries::credit_transactions::get_deducted_transactions::get_deducted_transactions(pool, "imageboard", &ids, Some("transaction_deduction"))
        .await
        .map_err(|_| WebhookErrorResponse{
            error: "Internal server error".to_string(),
            code: "INTERNAL_SERVER_ERROR".to_string(),
            details: Some("Failed to check duplicates".to_string()),
        })?;
    let processed_ids: std::collections::HashSet<Uuid> = already.iter().filter_map(|tx| tx.entity_id).collect();
    let unprocessed: Vec<&BillingTransactionRecord> = org_txs.into_iter()
        .filter(|t| Uuid::parse_str(&t.id).ok().map(|u| !processed_ids.contains(&u)).unwrap_or(true))
        .collect();

    // Process transactions (all transactions belong to the same collection with verified organization_id)
    let mut remaining = org_alloc.credits_remaining;
    let mut total_processed = 0;

    for t in unprocessed {

        let tx_credits = BigDecimal::from(t.amount_cents) / BigDecimal::from(CREDITS_TO_CENTS_RATIO);
        if remaining >= tx_credits {
            let logging_user_id = Uuid::parse_str(&t.user_id).unwrap_or_default();
            match crate::queries::organization_credit_allocation::deduct_organization_credits_with_transaction::deduct_organization_credits_with_transaction(
                pool,
                crate::queries::organization_credit_allocation::deduct_organization_credits_with_transaction::OrganizationCreditChangesParams{
                    user_id: logging_user_id,
                    organization_id,
                    credits_to_change: tx_credits.clone(),
                    action_source: "imageboard".to_string(),
                    action_type: "transaction_deduction".to_string(),
                    entity_id: Some(Uuid::parse_str(&t.id).unwrap()),
                }
            ).await {
                Ok(_) => {
                    total_processed += 1;
                    remaining -= tx_credits;
                }
                Err(_) => {
                    // Skip failed transactions, continue processing
                }
            }
        }
    }

    // Get final balance after processing
    let final_balance_cents = &remaining * BigDecimal::from(CREDITS_TO_CENTS_RATIO);
    let new_balance_cents = final_balance_cents.to_string().parse::<i32>().unwrap_or(0);

    Ok(BulkDeductCreditsLiteResponse{
        new_balance_cents,
        received_count: total_processed as i32,
    })
}

