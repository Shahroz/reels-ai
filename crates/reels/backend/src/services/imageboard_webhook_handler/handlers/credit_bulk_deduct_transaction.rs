use uuid::Uuid;
use sqlx::PgPool;
use bigdecimal::BigDecimal;

use crate::app_constants::credits_constants::CREDITS_TO_CENTS_RATIO;
use crate::schemas::imageboard_schemas::{WebhookErrorResponse, BulkDeductCreditsResponse, BillingTransactionRecord, FailedTransaction};

pub async fn handle_bulk_deduct(
    pool: &PgPool,
    transactions: &[BillingTransactionRecord],
) -> Result<BulkDeductCreditsResponse, WebhookErrorResponse> {

    if transactions.is_empty() {
        return Ok(BulkDeductCreditsResponse{
            succeeded_transaction_ids: vec![],
            total_processed: 0,
            total_failed: 0,
            failed: vec![],
            message: "No transactions to process".to_string(),
        });
    }

    let debit: Vec<&BillingTransactionRecord> = transactions.iter().filter(|t| t.transaction_type.to_lowercase() == "debit").collect();
    if debit.is_empty() {
        return Ok(BulkDeductCreditsResponse{
            succeeded_transaction_ids: vec![],
            total_processed: 0,
            total_failed: transactions.len(),
            failed: vec![],
            message: "No debit transactions found".to_string(),
        });
    }

    let ids: Vec<Uuid> = debit.iter().filter_map(|t| Uuid::parse_str(&t.id).ok()).collect();
    let already = crate::queries::credit_transactions::get_deducted_transactions::get_deducted_transactions(pool, "imageboard", &ids, Some("transaction_deduction")).await
        .map_err(|_| WebhookErrorResponse{ error: "Internal server error".to_string(), code: "INTERNAL_SERVER_ERROR".to_string(), details: Some("Failed to check duplicates".to_string()) })?;
    let processed_ids: std::collections::HashSet<Uuid> = already.iter().filter_map(|tx| tx.entity_id).collect();
    let unprocessed: Vec<&BillingTransactionRecord> = debit.into_iter().filter(|t| Uuid::parse_str(&t.id).ok().map(|u| !processed_ids.contains(&u)).unwrap_or(true)).collect();

    let mut succeeded_transaction_ids = Vec::new();
    let mut failed = Vec::new();

    use std::collections::HashMap;
    let mut by_org: HashMap<String, Vec<&BillingTransactionRecord>> = HashMap::new();
    for tx in &unprocessed { by_org.entry(tx.entity_id.clone()).or_default().push(tx); }

    for (org_id_str, org_txs) in by_org {
        let org_id = match Uuid::parse_str(&org_id_str) {
            Ok(u) => u,
            Err(_) => {
                failed.push(FailedTransaction{ reason: "Invalid Organization ID Format".to_string(), code: "INVALID_ORGANIZATION_ID".to_string(), transaction_id: None, organization_id: Some(org_id_str)});
                continue;
            }
        };

        let org_alloc = match crate::queries::organization_credit_allocation::get_organization_credit_allocation_by_org_id::get_organization_credit_allocation_by_org_id(pool, org_id).await {
            Ok(Some(a)) => a,
            Ok(None) => { failed.push(FailedTransaction{ reason: "Organization Credit Allocation Not Found".to_string(), code: "ORG_CREDIT_ALLOCATION_NOT_FOUND".to_string(), transaction_id: None, organization_id: Some(org_id.to_string())}); continue; }
            Err(_) => { failed.push(FailedTransaction{ reason: "Unknown Error".to_string(), code: "UNKNOWN_ERROR".to_string(), transaction_id: None, organization_id: Some(org_id.to_string())}); continue; }
        };

        let mut remaining = org_alloc.credits_remaining;
        for t in org_txs {
            let tx_credits = BigDecimal::from(t.amount_cents) / BigDecimal::from(CREDITS_TO_CENTS_RATIO);
            if remaining >= tx_credits {
                let logging_user_id = Uuid::parse_str(&t.user_id).unwrap_or_default();
                match crate::queries::organization_credit_allocation::deduct_organization_credits_with_transaction::deduct_organization_credits_with_transaction(pool, crate::queries::organization_credit_allocation::deduct_organization_credits_with_transaction::OrganizationCreditChangesParams{
                    user_id: logging_user_id,
                    organization_id: org_id,
                    credits_to_change: tx_credits.clone(),
                    action_source: "imageboard".to_string(),
                    action_type: "transaction_deduction".to_string(),
                    entity_id: Some(Uuid::parse_str(&t.id).unwrap()),
                }).await {
                    Ok(_) => { succeeded_transaction_ids.push(t.id.clone()); remaining -= tx_credits; }
                    Err(_) => { failed.push(FailedTransaction{ reason: "Unknown Error".to_string(), code: "UNKNOWN_ERROR".to_string(), transaction_id: Some(t.id.clone()), organization_id: Some(org_id.to_string())}); }
                }
            } else {
                failed.push(FailedTransaction{ reason: "Insufficient Balance".to_string(), code: "INSUFFICIENT_BALANCE".to_string(), transaction_id: Some(t.id.clone()), organization_id: Some(org_id.to_string())});
            }
        }
    }

    let total_processed = succeeded_transaction_ids.len();
    Ok(BulkDeductCreditsResponse{
        succeeded_transaction_ids,
        total_processed,
        total_failed: failed.len(),
        failed,
        message: format!("Successfully processed {} transaction(s)", total_processed),
    })
}


