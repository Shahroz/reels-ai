//! Stripe webhook event handlers.
//!
//! This module contains individual handlers for different Stripe webhook events.
//! Each handler is responsible for processing a specific event type and follows
//! the one-file-per-item pattern.

pub mod checkout_session_completed;
pub mod subscription_created;
pub mod subscription_updated;
pub mod subscription_deleted;
pub mod invoice_payment_succeeded;
pub mod invoice_payment_failed;
pub mod invoice_created;
pub mod invoice_finalized;
pub mod invoice_paid;
pub mod handle_organization_invoice_paid;
pub mod product_updated;

// Re-export all handlers for convenience
pub use checkout_session_completed::handle_checkout_session_completed_event;
pub use subscription_created::handle_subscription_created_event;
pub use subscription_updated::handle_subscription_updated_event;
pub use subscription_deleted::handle_subscription_deleted_event;
pub use invoice_payment_succeeded::handle_invoice_payment_succeeded_event;
pub use invoice_payment_failed::handle_invoice_payment_failed_event;
pub use invoice_created::handle_invoice_created_event;
pub use invoice_finalized::handle_invoice_finalized_event;
pub use invoice_paid::handle_invoice_paid_event;
pub use handle_organization_invoice_paid::handle_organization_invoice_paid;
pub use product_updated::handle_product_updated_event;
