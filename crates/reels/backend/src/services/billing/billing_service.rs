use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::instrument;
use uuid::Uuid;

use crate::db::users::update_user_stripe_id;
use crate::app_constants::credits_constants::FREE_CREDITS;
use crate::schemas::user_credit_allocation_schemas::StripePlanType;
use crate::services::billing::billing_service_trait::BillingServiceTrait;
use crate::queries::user_credit_allocation::{create_or_update_user_credit_allocation_with_transaction, get_user_credit_allocation_by_user_id};
use crate::services::billing::stripe_client::{StripeCancellationDetails, StripeClient, StripeProductWithPrices, StripePrice, StripePlanWithProduct, StripeCustomer, StripeSubscriptionList};

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CheckoutSessionRequest {
    pub price_id: String,
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CheckoutSessionResponse {
    pub session_id: String,
    pub session_url: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CustomerPortalResponse {
    pub portal_url: String,
}

pub struct BillingService {
    stripe_client: StripeClient,
}

impl BillingService {
    pub fn new() -> Result<Self> {
        let stripe_client = StripeClient::new()?;
        Ok(BillingService { stripe_client })
    }

    /// Create a new billing service with a specific Stripe client
    /// This method enables dependency injection for better testing
    pub fn new_with_client(stripe_client: StripeClient) -> Result<Self> {
        Ok(BillingService { stripe_client })
    }

    /// Fetch Stripe products tagged with `product_type: real_estate or narrativ` and optionally filtered by meta_product_type
    #[instrument(skip(self))]
    pub async fn get_products(&self, meta_product_type: Option<&str>, active: Option<bool>) -> Result<Vec<StripeProductWithPrices>> {
        let mut products = Vec::new();
        
        let product_list = self.stripe_client.list_products(active, None).await?;
        
        for product in product_list.data {
            // Check if product has the specified product_type in metadata
            if let Some(metadata) = product.metadata.as_object() {
                // If meta_product_type is provided, filter by it; otherwise include all products
                if let Some(expected_product_type) = meta_product_type {
                    // Check if the metadata contains the expected product_type value
                    if metadata.get("product_type") != Some(&serde_json::Value::String(expected_product_type.to_string())) {
                        continue; // Skip products that don't match the expected product_type
                    }
                }
                
                // Fetch prices for this product
                let price_list = self.stripe_client.list_prices(Some(&product.id), active).await?;
                log::info!("Fetched {} prices for product {}", price_list.data.len(), product.id);
                
                let prices: Vec<StripePrice> = price_list.data
                    .into_iter()
                    .filter(|price| price.active && price.product == product.id) // Only include active prices for this specific product
                    .map(|price| StripePrice {
                        id: price.id,
                        object: price.object,
                        active: price.active,
                        billing_scheme: price.billing_scheme,
                        created: price.created,
                        currency: price.currency,
                        currency_options: price.currency_options,
                        custom_unit_amount: price.custom_unit_amount,
                        livemode: price.livemode,
                        lookup_key: price.lookup_key,
                        metadata: price.metadata,
                        nickname: price.nickname,
                        product: price.product,
                        recurring: price.recurring,
                        tax_behavior: price.tax_behavior,
                        tiers_mode: price.tiers_mode,
                        tiers: price.tiers,
                        transform_quantity: price.transform_quantity,
                        unit_amount: price.unit_amount,
                        unit_amount_decimal: price.unit_amount_decimal,
                        price_type: price.price_type,
                    }).collect();
                
                log::info!("Filtered to {} active prices for product {}", prices.len(), product.id);
                
                products.push(StripeProductWithPrices {
                    id: product.id,
                    name: product.name,
                    description: product.description,
                    metadata: product.metadata,
                    default_price: product.default_price,
                    active: product.active,
                    created: product.created,
                    updated: product.updated,
                    prices,
                    object: "product".to_string(),
                    marketing_features: product.marketing_features,
                    images: product.images,
                    package_dimensions: product.package_dimensions,
                    shippable: product.shippable,
                    statement_descriptor: product.statement_descriptor,
                    tax_code: product.tax_code,
                    unit_label: product.unit_label,
                    url: product.url,
                });
            }
        }
        
        Ok(products)
    }

    /// Get a specific product by ID
    #[instrument(skip(self))]
    pub async fn get_product(&self, product_id: &str, with_prices: bool) -> Result<StripeProductWithPrices> {
        let product = self.stripe_client.get_product(product_id).await?;
        let product_type = std::env::var("STRIPE_METADATA_PRODUCT_TYPE").unwrap_or_default();
        
        // Check if product has the specified product_type in metadata
        if let Some(metadata) = product.metadata.as_object() {
            // Check if the metadata contains the expected product_type value
            if metadata.get("product_type") != Some(&serde_json::Value::String(product_type.clone())) {
                return Err(anyhow::anyhow!("Product does not match expected product_type, expected: {}, got: {}", product_type, metadata.get("product_type").unwrap_or(&serde_json::Value::String("unknown".to_string()))));
            }

            // Fetch prices for this product if requested
            if with_prices {
                let price_list = self.stripe_client.list_prices(Some(&product.id), Some(true)).await?;
                log::info!("Fetched {} prices for product {}", price_list.data.len(), product.id);
                
                let prices: Vec<StripePrice> = price_list.data
                    .into_iter()
                    .filter(|price| price.active && price.product == product.id) // Only include active prices for this specific product
                    .map(|price| StripePrice {
                        id: price.id,
                        object: price.object,
                        active: price.active,
                        billing_scheme: price.billing_scheme,
                        created: price.created,
                        currency: price.currency,
                        currency_options: price.currency_options,
                        custom_unit_amount: price.custom_unit_amount,
                        livemode: price.livemode,
                        lookup_key: price.lookup_key,
                        metadata: price.metadata,
                        nickname: price.nickname,
                        product: price.product,
                        recurring: price.recurring,
                        tax_behavior: price.tax_behavior,
                        tiers_mode: price.tiers_mode,
                        tiers: price.tiers,
                        transform_quantity: price.transform_quantity,
                        unit_amount: price.unit_amount,
                        unit_amount_decimal: price.unit_amount_decimal,
                        price_type: price.price_type,
                    }).collect();
                
                log::info!("Filtered to {} active prices for product {}", prices.len(), product.id);
                
                return Ok(StripeProductWithPrices {
                    id: product.id,
                    name: product.name,
                    description: product.description,
                    metadata: product.metadata,
                    default_price: product.default_price.clone(),
                    active: product.active,
                    created: product.created,
                    updated: product.updated,
                    prices,
                    object: "product".to_string(),
                    marketing_features: product.marketing_features,
                    images: product.images,
                    package_dimensions: product.package_dimensions,
                    shippable: product.shippable,
                    statement_descriptor: product.statement_descriptor,
                    tax_code: product.tax_code,
                    unit_label: product.unit_label,
                    url: product.url,
                });
            }
        }
        
        // Return product without prices if with_prices is false or metadata check failed
        Ok(StripeProductWithPrices {
            id: product.id,
            name: product.name,
            description: product.description,
            metadata: product.metadata,
            default_price: product.default_price.clone(),
            active: product.active,
            created: product.created,
            updated: product.updated,
            prices: vec![],
            object: "product".to_string(),
            marketing_features: product.marketing_features,
            images: product.images,
            package_dimensions: product.package_dimensions,
            shippable: product.shippable,
            statement_descriptor: product.statement_descriptor,
            tax_code: product.tax_code,
            unit_label: product.unit_label,
            url: product.url,
        })
    }

    /// Get a specific price by ID
    #[instrument(skip(self))]
    pub async fn get_price(&self, price_id: &str) -> Result<StripePrice> {
        self.stripe_client.get_price(price_id).await
    }

    /// Fetch Stripe plans with product details
    #[instrument(skip(self))]
    pub async fn get_plans(
        &self,
        meta_product_type: Option<&str>,
        active: Option<bool>,
    ) -> Result<Vec<StripePlanWithProduct>> {
        let plans_response = self.stripe_client.list_plans(active, None).await?;
        
        let mut plans_with_products = Vec::new();
        
        for plan in plans_response.data {
            // Fetch product details for this plan
            let mut product = self.stripe_client.get_product(&plan.product).await?;
            if let Some(meta_product_type) = meta_product_type {
                if product.metadata.get("product_type") != Some(&serde_json::Value::String(meta_product_type.to_string())) {
                    continue;
                }
            }

            // Fetch prices for this product with the plan's billing scheme
            let prices: Vec<StripePrice> = self.stripe_client.list_prices(Some(&plan.product), active).await?.data;
            product.prices = Some(prices);
            let plan_with_product = StripePlanWithProduct {
                plan,
                product_details: product,
            };
            
            plans_with_products.push(plan_with_product);
        }
        
        Ok(plans_with_products)
    }

    /// Create a Stripe checkout session
    #[instrument(skip(self, pool))]
    pub async fn create_checkout_session(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        user_email: &str,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
        mode: &str,
        dub_id: Option<&str>,
    ) -> Result<CheckoutSessionResponse> {
        self.create_checkout_session_with_context(
            pool,
            user_id,
            user_email,
            price_id,
            success_url,
            cancel_url,
            mode,
            dub_id,
            "user",
            None,
        ).await
    }

    /// Create a Stripe checkout session with customer type and optional organization context
    #[instrument(skip(self, pool))]
    pub async fn create_checkout_session_with_context(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        user_email: &str,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
        mode: &str,
        dub_id: Option<&str>,
        customer_type: &str,
        organization_id: Option<Uuid>,
    ) -> Result<CheckoutSessionResponse> {
        // Get existing customer ID if this is an organization or user with existing customer
        let existing_customer_id = if let Some(org_id) = organization_id {
            // Get organization's existing Stripe customer ID if it has one
            match crate::queries::organizations::find_organization_by_id(pool, org_id).await? {
                Some(org) => org.stripe_customer_id,
                None => None,
            }
        } else {
            // Get user's existing Stripe customer ID if they have one
            match crate::queries::webhooks::get_user_stripe_customer_id(pool, user_id).await? {
                Some(id) => Some(id),
                None => None,
            }
        };

        // Get or create Stripe customer with the specified type
        let customer = self.get_or_create_customer_with_type(
            user_email, 
            customer_type, 
            existing_customer_id.as_deref()
        ).await?;
        
        // Update customer ID in database based on context (only if new customer was created)
        if existing_customer_id.is_none() || existing_customer_id.as_ref() != Some(&customer.id) {
            if let Some(org_id) = organization_id {
                // Update organization's Stripe customer ID
                crate::queries::organizations::update_organization_stripe_customer_id(pool, org_id, &customer.id).await?;
                log::info!("Updated organization {} with Stripe customer {}", org_id, customer.id);
            } else {
                // Update user's Stripe customer ID
                update_user_stripe_id(pool, user_id, &customer.id).await?;
                log::info!("Updated user {} with Stripe customer {}", user_id, customer.id);
            }
        } else {
            log::info!("Using existing Stripe customer {} for {} {}", 
                customer.id, 
                customer_type, 
                organization_id.map(|id| id.to_string()).unwrap_or_else(|| user_id.to_string())
            );
        }
        
        // Create line items
        let mut line_item = HashMap::new();
        line_item.insert("price".to_string(), price_id.to_string());
        line_item.insert("quantity".to_string(), "1".to_string());
        
        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("user_id".to_string(), user_id.to_string());
        metadata.insert("price_id".to_string(), price_id.to_string());
        metadata.insert("customer_type".to_string(), customer_type.to_string());
        
        // Add organization_id to metadata if present
        if let Some(org_id) = organization_id {
            metadata.insert("organization_id".to_string(), org_id.to_string());
            log::info!("Creating organization checkout session for org {}", org_id);
        }
        
        // Add official Dub integration metadata
        metadata.insert("dubCustomerId".to_string(), user_id.to_string());
        if let Some(dub_id) = dub_id {
            metadata.insert("dubClickId".to_string(), dub_id.to_string());
            log::info!("Adding Dub attribution metadata - dubCustomerId: {}, dubClickId: {}", user_id, dub_id);
        } else {
            log::info!("Adding Dub attribution metadata - dubCustomerId: {} (no click ID)", user_id);
        }
        
        let session = self.stripe_client.create_checkout_session(
            success_url,
            cancel_url,
            Some(&customer.id),
            vec![line_item],
            mode,
            metadata,
        ).await?;
        
        // Store checkout session in database
        crate::queries::billing::checkout_sessions::create_checkout_session(pool, user_id, &session.id, price_id).await?;
        
        Ok(CheckoutSessionResponse {
            session_id: session.id,
            session_url: session.url.unwrap_or_default(),
        })
    }

    /// Create a customer portal session
    #[instrument(skip(self, pool))]
    pub async fn create_customer_portal_session(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        return_url: &str,
    ) -> Result<CustomerPortalResponse> {
        // Get user's Stripe customer ID
        let stripe_customer_id = crate::queries::billing::users::get_user_stripe_customer_id(pool, user_id).await?;
        let customer_id = stripe_customer_id
            .ok_or_else(|| anyhow::anyhow!("User has no Stripe customer ID"))?;
        
        // Create portal session
        let session = self.stripe_client.create_customer_portal_session(&customer_id, return_url).await?;
        
        Ok(CustomerPortalResponse {
            portal_url: session.url,
        })
    }

    /// Create a free subscription for a user
    /// This method now only handles credit allocation, not Stripe subscriptions
    #[instrument(skip(self, pool))]
    pub async fn create_free_subscription(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        user_email: &str,
    ) -> Result<()> {
        // Check if user already has a credit allocation (free user)
        let existing_allocation = get_user_credit_allocation_by_user_id(pool, user_id).await?;
        if existing_allocation.is_some() {
            log::info!("User already has a credit allocation, skipping creation");
            return Ok(());
        }

        // For free users, we use a default credit limit based on FREE_CREDITS constant
        let credit_limit = FREE_CREDITS as i32;
        
        // Insert record in user_credit_allocation table
        let _user_credit_allocation = create_or_update_user_credit_allocation_with_transaction(
            pool,
            user_id,
            StripePlanType::Free,
            2,
            FREE_CREDITS, // one time credits for free plan
            credit_limit,
            None,
        ).await.map_err(|e| {
            log::error!("Failed to create user credit allocation for user {}: {}", user_id, e);
            e
        })?;
        log::info!("Successfully created user credit allocation for user: {}", user_id);

        // Get or create Stripe customer
        let customer = self.get_or_create_customer(user_email).await?;
        
        // Update user's Stripe customer ID in database
        update_user_stripe_id(pool, user_id, &customer.id).await?;

        Ok(())
    }

    /// Get or create a Stripe customer with specified customer type
    /// 
    /// For organizations: This method supports lazy creation. If the organization already has
    /// a stripe_customer_id, it will be retrieved from Stripe rather than creating a new one.
    /// This prevents duplicate Stripe customers for the same organization.
    /// For users: Reuses existing customers by email.
    #[instrument(skip(self, email, customer_id))]
    async fn get_or_create_customer_with_type(&self, email: &str, customer_type: &str, customer_id: Option<&str>) -> Result<StripeCustomer> {
        let stripe_metadata_product_type = std::env::var("STRIPE_METADATA_PRODUCT_TYPE").unwrap_or_default();
        if stripe_metadata_product_type.is_empty() {
            let err_msg = format!("[BILLING] STRIPE_METADATA_PRODUCT_TYPE environment variable not set");
            log::error!("{}", err_msg);
            return Err(anyhow::anyhow!(err_msg));
        }

        log::info!("[BILLING] Looking up customer for email: {} with type: {} and customer_id: {:?}", email, customer_type, customer_id);
        
        // For organizations, check if they already have a Stripe customer ID
        if customer_type == "organization" {
            // If the organization already has a stripe_customer_id, retrieve that customer
            if let Some(existing_customer_id) = customer_id {
                log::info!("[BILLING] Organization already has Stripe customer: {}, retrieving it", existing_customer_id);
                match self.stripe_client.get_customer(existing_customer_id).await {
                    Ok(customer) => {
                        log::info!("[BILLING] Successfully retrieved existing organization customer: {}", customer.id);
                        return Ok(customer);
                    }
                    Err(e) => {
                        log::warn!("[BILLING] Failed to retrieve existing customer {}, will create new one: {}", existing_customer_id, e);
                        // Fall through to create a new customer
                    }
                }
            }
            
            // Create a new customer for the organization (lazy creation)
            log::info!("[BILLING] Creating new customer for organization (lazy creation)");
            
            let mut metadata = HashMap::new();
            metadata.insert("product_type".to_string(), stripe_metadata_product_type);
            metadata.insert("customer_type".to_string(), customer_type.to_string());
            
            let customer = match self.stripe_client.create_customer(email, metadata).await {
                Ok(customer) => {
                    log::info!("[BILLING] Successfully created new organization customer: {} for email: {}", customer.id, email);
                    customer
                },
                Err(e) => {
                    log::error!("[BILLING] Failed to create organization customer for email {email}: {e}");
                    return Err(e);
                }
            };
            
            return Ok(customer);
        }
        
        // For users, try to find existing customer by email
        let customers = match self.stripe_client.list_customers_by_email(email).await {
            Ok(customers) => {
                log::info!("[BILLING] Found {} existing customers for email: {}", customers.data.len(), email);
                customers
            },
            Err(e) => {
                log::error!("[BILLING] Failed to search for existing customer with email {email}: {e}");
                return Err(e);
            }
        };
        
        // Only reuse customer if it's a USER customer (check metadata)
        for customer in customers.data.iter() {
            let is_user_customer = customer.metadata
                .get("customer_type")
                .and_then(|v| v.as_str())
                .map(|t| t == "user")
                .unwrap_or(true); // Default to user for backwards compatibility
            
            if is_user_customer {
                log::info!("[BILLING] Using existing user customer: {} for email: {}", customer.id, email);
                return Ok(customer.clone());
            }
        }
        
        // Create new user customer
        log::info!("[BILLING] Creating new user customer for email: {}", email);
        let mut metadata = HashMap::new();
        metadata.insert("product_type".to_string(), stripe_metadata_product_type);
        metadata.insert("customer_type".to_string(), customer_type.to_string());
        
        let customer = match self.stripe_client.create_customer(email, metadata).await {
            Ok(customer) => {
                log::info!("[BILLING] Successfully created new user customer: {} for email: {}", customer.id, email);
                customer
            },
            Err(e) => {
                log::error!("[BILLING] Failed to create user customer for email {email}: {e}");
                return Err(e);
            }
        };
        
        Ok(customer)
    }

    /// Get or create a Stripe customer (defaults to user type)
    #[instrument(skip(self, email))]
    async fn get_or_create_customer(&self, email: &str) -> Result<StripeCustomer> {
        self.get_or_create_customer_with_type(email, "user", None).await
    }

    /// Update checkout session status
    #[instrument(skip(self, pool))]
    pub async fn update_checkout_session_status(
        &self,
        pool: &PgPool,
        stripe_checkout_id: &str,
        status: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        crate::queries::billing::checkout_sessions::update_checkout_session_status(pool, stripe_checkout_id, status, metadata).await?;
        
        log::info!("Updated checkout session: {} status to: {}", stripe_checkout_id, status);
        Ok(())
    }

    /// Activate user subscription after successful payment
    #[instrument(skip(self, pool))]
    pub async fn activate_user_subscription(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        stripe_customer_id: &str,
    ) -> Result<()> {
        crate::queries::billing::users::activate_user_subscription(pool, user_id, stripe_customer_id).await?;
        
        log::info!("Activated subscription and ended trial for user: {}", user_id);
        Ok(())
    }

    /// Cancel a Stripe subscription and update local database
    /// Based on Stripe API: https://docs.stripe.com/api/subscriptions/cancel
    #[instrument(skip(self, _pool))]
    async fn cancel_subscription(
        &self,
        _pool: &PgPool,
        subscription_id: &str,
        cancellation_details: Option<StripeCancellationDetails>,
        invoice_now: Option<bool>,
        prorate: Option<bool>,
    ) -> Result<()> {
        log::info!("Canceling Stripe subscription: {}", subscription_id);

        // Prepare cancellation details for Stripe API
        let mut _cancellation_details = HashMap::new();
        if let Some(details) = cancellation_details {
            if let Some(comment) = details.comment {
                _cancellation_details.insert("comment".to_string(), comment);
            }
            if let Some(feedback) = details.feedback {
                _cancellation_details.insert("feedback".to_string(), feedback.as_str().to_string());
            }
        }

        // Cancel the subscription in Stripe
        let canceled_subscription = self.stripe_client
            .cancel_subscription(
                subscription_id,
                Some(_cancellation_details),
                invoice_now,
                prorate,
            )
            .await?;

        log::info!(
            "Successfully canceled Stripe subscription: {} (status: {})", 
            subscription_id, 
            canceled_subscription.status
        );

        Ok(())
    }
    
    /// Get subscriptions by customer ID, filtered by products matching STRIPE_METADATA_PRODUCT_TYPE
    /// Based on Stripe API: https://docs.stripe.com/api/subscriptions/list
    #[instrument(skip(self))]
    async fn get_subscriptions_by_customer(&self, customer_id: &str) -> Result<StripeSubscriptionList> {
        // Get the expected product type from environment
        let stripe_metadata_product_type = std::env::var("STRIPE_METADATA_PRODUCT_TYPE").unwrap_or_default();
        if stripe_metadata_product_type.is_empty() {
            log::warn!("STRIPE_METADATA_PRODUCT_TYPE environment variable not set, returning empty subscription list");
            return Ok(StripeSubscriptionList {
                object: "list".to_string(),
                data: vec![],
                has_more: false,
                url: format!("/v1/subscriptions?customer={}", customer_id),
            });
        }

        // Get all products matching the product type
        let products = self.get_products(Some(&stripe_metadata_product_type), Some(true)).await?;
        if products.is_empty() {
            log::info!("No products found for product_type: {}, returning empty subscription list", stripe_metadata_product_type);
            return Ok(StripeSubscriptionList {
                object: "list".to_string(),
                data: vec![],
                has_more: false,
                url: format!("/v1/subscriptions?customer={}", customer_id),
            });
        }

        // Extract product IDs from the filtered products
        let valid_product_ids: std::collections::HashSet<String> = products
            .iter()
            .map(|p| p.id.clone())
            .collect();

        // Get all subscriptions for the customer
        let mut all_subscriptions = self.stripe_client.get_subscriptions_by_customer(customer_id).await?;

        // Filter subscriptions by product_ids
        all_subscriptions.data.retain(|subscription| {
            subscription.items.data.iter().any(|item| {
                valid_product_ids.contains(&item.price.product)
            })
        });

        Ok(all_subscriptions)
    }
}

#[async_trait::async_trait]
impl BillingServiceTrait for BillingService {
    async fn get_products(&self, meta_product_type: Option<&str>, active: Option<bool>) -> Result<Vec<StripeProductWithPrices>> {
        self.get_products(meta_product_type, active).await
    }

    async fn get_product(&self, product_id: &str, with_prices: bool) -> Result<StripeProductWithPrices> {
        self.get_product(product_id, with_prices).await
    }

    async fn get_price(&self, price_id: &str) -> Result<StripePrice> {
        self.get_price(price_id).await
    }

    async fn get_plans(&self, meta_product_type: Option<&str>, active: Option<bool>) -> Result<Vec<StripePlanWithProduct>> {
        self.get_plans(meta_product_type, active).await
    }

    async fn create_checkout_session_with_context(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        user_email: &str,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
        mode: &str,
        dub_id: Option<&str>,
        customer_type: &str,
        organization_id: Option<Uuid>,
    ) -> Result<CheckoutSessionResponse> {
        self.create_checkout_session_with_context(
            pool,
            user_id,
            user_email,
            price_id,
            success_url,
            cancel_url,
            mode,
            dub_id,
            customer_type,
            organization_id,
        ).await
    }

    async fn create_free_subscription(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        user_email: &str,
    ) -> Result<()> {
        self.create_free_subscription(pool, user_id, user_email).await
    }

    async fn create_checkout_session(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        user_email: &str,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
        mode: &str,
        dub_id: Option<&str>,
    ) -> Result<CheckoutSessionResponse> {
        self.create_checkout_session(pool, user_id, user_email, price_id, success_url, cancel_url, mode, dub_id).await
    }

    async fn create_customer_portal_session(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        return_url: &str,
    ) -> Result<CustomerPortalResponse> {
        self.create_customer_portal_session(pool, user_id, return_url).await
    }

    async fn update_checkout_session_status(
        &self,
        pool: &PgPool,
        stripe_checkout_id: &str,
        status: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        self.update_checkout_session_status(pool, stripe_checkout_id, status, metadata).await
    }

    async fn activate_user_subscription(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        stripe_customer_id: &str,
    ) -> Result<()> {
        self.activate_user_subscription(pool, user_id, stripe_customer_id).await
    }

    async fn cancel_subscription(
        &self,
        pool: &PgPool,
        subscription_id: &str,
        cancellation_details: Option<StripeCancellationDetails>,
        invoice_now: Option<bool>,
        prorate: Option<bool>,
    ) -> Result<()> {
        self.cancel_subscription(pool, subscription_id, cancellation_details, invoice_now, prorate).await
    }

    async fn get_subscriptions_by_customer(&self, customer_id: &str) -> Result<StripeSubscriptionList> {
        self.get_subscriptions_by_customer(customer_id).await
    }

    async fn get_customer(&self, customer_id: &str) -> Result<StripeCustomer> {
        self.stripe_client.get_customer(customer_id).await
    }

    async fn update_customer_email(&self, customer_id: &str, new_email: &str) -> Result<StripeCustomer> {
        self.stripe_client.update_customer_email(customer_id, new_email).await
    }
}
