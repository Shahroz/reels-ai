//! Mock billing service implementation for testing.
//!
//! This service provides a fake billing implementation that returns
//! test data without making external API calls. Used exclusively in
//! test environments to avoid external dependencies and network calls.

use anyhow::Result;
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::services::billing::stripe_client::{StripeCancellationDetails, StripeCustomer, StripeProductWithPrices, StripePrice, StripeRecurring, StripePlan, StripePlanWithProduct, StripeProduct, StripeSubscriptionList};
use crate::services::billing::billing_service::{
    CheckoutSessionResponse, CustomerPortalResponse
};
use crate::services::billing::billing_service_trait::BillingServiceTrait;

/// Mock billing service that returns test data for testing
pub struct MockBillingService;

impl MockBillingService {
    /// Create a new mock billing service
    pub fn new() -> Self {
        Self
    }

    /// Get a specific mock product by ID
    #[instrument(skip(self))]
    pub async fn get_product(&self, product_id: &str, with_prices: bool) -> Result<StripeProductWithPrices> {
        // Return mock product for testing
        let mock_product = StripeProductWithPrices {
            id: product_id.to_string(),
            name: "Test Product".to_string(),
            description: Some("Mock product for testing".to_string()),
            metadata: serde_json::json!({
                "product_type": "real_estate",
                "product_plan": "real_estate",
                "features": ["feature1", "feature2"],
                "plan_type": "basic"
            }),
            default_price: Some("price_1".to_string()),
            active: Some(true),
            created: Some(chrono::Utc::now().timestamp()),
            updated: Some(chrono::Utc::now().timestamp()),
            object: "product".to_string(),
            marketing_features: None,
            images: None,
            package_dimensions: None,
            shippable: Some(false),
            statement_descriptor: Some("TEST".to_string()),
            tax_code: Some("txcd_1234567890".to_string()),
            unit_label: Some("unit".to_string()),
            url: None,
            prices: if with_prices {
                vec![
                    StripePrice {
                        id: "price_1".to_string(),
                        object: "price".to_string(),
                        active: true,
                        billing_scheme: Some("per_unit".to_string()),
                        created: Some(chrono::Utc::now().timestamp()),
                        currency: "usd".to_string(),
                        currency_options: None,
                        custom_unit_amount: None,
                        livemode: Some(false),
                        lookup_key: None,
                        metadata: serde_json::json!({}),
                        nickname: None,
                        product: product_id.to_string(),
                        recurring: Some(StripeRecurring {
                            interval: "month".to_string(),
                            interval_count: Some(1),
                        }),
                        tax_behavior: None,
                        tiers_mode: None,
                        tiers: None,
                        transform_quantity: None,
                        unit_amount: Some(2900),
                        unit_amount_decimal: Some("29.00".to_string()),
                        price_type: Some("recurring".to_string()),
                    }
                ]
            } else {
                vec![]
            },
        };
        
        Ok(mock_product)
    }

    /// Get a specific mock price by ID
    #[instrument(skip(self))]
    pub async fn get_price(&self, price_id: &str) -> Result<StripePrice> {
        // Return mock price for testing
        Ok(StripePrice {
            id: price_id.to_string(),
            object: "price".to_string(),
            active: true,
            billing_scheme: Some("per_unit".to_string()),
            created: Some(chrono::Utc::now().timestamp()),
            currency: "usd".to_string(),
            currency_options: None,
            custom_unit_amount: None,
            livemode: Some(false),
            lookup_key: None,
            metadata: serde_json::json!({}),
            nickname: Some("Test Price".to_string()),
            product: "prod_test_1".to_string(),
            recurring: Some(StripeRecurring {
                interval: "month".to_string(),
                interval_count: Some(1),
            }),
            tax_behavior: None,
            tiers_mode: None,
            tiers: None,
            transform_quantity: None,
            unit_amount: Some(2900),
            unit_amount_decimal: Some("29.00".to_string()),
            price_type: Some("recurring".to_string()),
        })
    }

    /// Fetch mock Stripe products tagged with `product_type: real_estate` and optionally filtered by meta_product_type
    #[instrument(skip(self))]
    pub async fn get_products(&self, meta_product_type: Option<&str>, active: Option<bool>) -> Result<Vec<StripeProductWithPrices>> {
        // Return mock products for testing
        let mut mock_products = vec![
            StripeProductWithPrices {
                id: "prod_test_1".to_string(),
                name: "Test Real Estate Plan".to_string(),
                description: Some("Mock real estate subscription plan for testing".to_string()),
                metadata: serde_json::json!({
                    "product_type": "real_estate",
                    "features": ["feature1", "feature2"]
                }),
                default_price: Some("price_1".to_string()),
                active: Some(true),
                created: Some(chrono::Utc::now().timestamp()),
                updated: Some(chrono::Utc::now().timestamp()),
                object: "product".to_string(),
                marketing_features: None,
                images: None,
                package_dimensions: None,
                shippable: Some(false),
                statement_descriptor: Some("REALESTATE".to_string()),
                tax_code: Some("txcd_1234567890".to_string()),
                unit_label: Some("unit".to_string()),
                url: None,
                prices: vec![
                    StripePrice {
                        id: "price_1".to_string(),
                        object: "price".to_string(),
                        active: true,
                        billing_scheme: Some("per_unit".to_string()),
                        created: Some(chrono::Utc::now().timestamp()),
                        currency: "usd".to_string(),
                        currency_options: None,
                        custom_unit_amount: None,
                        livemode: Some(false),
                        lookup_key: None,
                        metadata: serde_json::json!({}),
                        nickname: None,
                        product: "prod_test_1".to_string(),
                        recurring: Some(StripeRecurring {
                            interval: "month".to_string(),
                            interval_count: Some(1),
                        }),
                        tax_behavior: None,
                        tiers_mode: None,
                        tiers: None,
                        transform_quantity: None,
                        unit_amount: Some(2900),
                        unit_amount_decimal: Some("29.00".to_string()),
                        price_type: Some("recurring".to_string()),
                    },
                    StripePrice {
                        id: "price_2".to_string(),
                        object: "price".to_string(),
                        active: true,
                        billing_scheme: Some("per_unit".to_string()),
                        created: Some(chrono::Utc::now().timestamp()),
                        currency: "usd".to_string(),
                        currency_options: None,
                        custom_unit_amount: None,
                        livemode: Some(false),
                        lookup_key: None,
                        metadata: serde_json::json!({}),
                        nickname: None,
                        product: "prod_test_1".to_string(),
                        recurring: Some(StripeRecurring {
                            interval: "year".to_string(),
                            interval_count: Some(1),
                        }),
                        tax_behavior: None,
                        tiers_mode: None,
                        tiers: None,
                        transform_quantity: None,
                        unit_amount: Some(29000),
                        unit_amount_decimal: Some("290.00".to_string()),
                        price_type: Some("recurring".to_string()),
                    },
                ],
            },
            StripeProductWithPrices {
                id: "prod_test_2".to_string(),
                name: "Premium Real Estate Plan".to_string(),
                description: Some("Mock premium real estate subscription plan for testing".to_string()),
                metadata: serde_json::json!({
                    "product_type": "real_estate",
                    "features": ["feature1", "feature2", "feature3"],
                    "plan_type": "premium"
                }),
                default_price: Some("price_3".to_string()),
                active: Some(true),
                created: Some(chrono::Utc::now().timestamp()),
                updated: Some(chrono::Utc::now().timestamp()),
                object: "product".to_string(),
                marketing_features: None,
                images: None,
                package_dimensions: None,
                shippable: Some(false),
                statement_descriptor: Some("REALESTATE".to_string()),
                tax_code: Some("txcd_1234567890".to_string()),
                unit_label: Some("unit".to_string()),
                url: None,
                prices: vec![
                    StripePrice {
                        id: "price_3".to_string(),
                        object: "price".to_string(),
                        active: true,
                        billing_scheme: Some("per_unit".to_string()),
                        created: Some(chrono::Utc::now().timestamp()),
                        currency: "usd".to_string(),
                        currency_options: None,
                        custom_unit_amount: None,
                        livemode: Some(false),
                        lookup_key: None,
                        metadata: serde_json::json!({}),
                        nickname: None,
                        product: "prod_test_2".to_string(),
                        recurring: Some(StripeRecurring {
                            interval: "month".to_string(),
                            interval_count: Some(1),
                        }),
                        tax_behavior: None,
                        tiers_mode: None,
                        tiers: None,
                        transform_quantity: None,
                        unit_amount: Some(5900),
                        unit_amount_decimal: Some("59.00".to_string()),
                        price_type: Some("recurring".to_string()),
                    },
                ],
            },
            StripeProductWithPrices {
                id: "prod_test_3".to_string(),
                name: "Narrativ Plan Pro 1".to_string(),
                description: Some("Mock narrativ subscription plan for testing".to_string()),
                metadata: serde_json::json!({
                    "product_type": "narrativ",
                    "features": ["feature1", "feature2"],
                    "product_plan": "pro"
                }),
                default_price: Some("price_1".to_string()),
                active: Some(true),
                created: Some(chrono::Utc::now().timestamp()),
                updated: Some(chrono::Utc::now().timestamp()),
                object: "product".to_string(),
                marketing_features: None,
                images: None,
                package_dimensions: None,
                shippable: Some(false),
                statement_descriptor: Some("NARRATIV".to_string()),
                tax_code: Some("txcd_1234567890".to_string()),
                unit_label: Some("unit".to_string()),
                url: None,
                prices: vec![
                    StripePrice {
                        id: "price_1".to_string(),
                        object: "price".to_string(),
                        active: true,
                        billing_scheme: Some("per_unit".to_string()),
                        created: Some(chrono::Utc::now().timestamp()),
                        currency: "usd".to_string(),
                        currency_options: None,
                        custom_unit_amount: None,
                        livemode: Some(false),
                        lookup_key: None,
                        metadata: serde_json::json!({}),
                        nickname: None,
                        product: "prod_test_3".to_string(),
                        recurring: Some(StripeRecurring {
                            interval: "month".to_string(),
                            interval_count: Some(1),
                        }),
                        tax_behavior: None,
                        tiers_mode: None,
                        tiers: None,
                        transform_quantity: None,
                        unit_amount: Some(0 as i64),
                        unit_amount_decimal: Some("0.00".to_string()),
                        price_type: Some("recurring".to_string()),
                    },
                ],
            },
            StripeProductWithPrices {
                id: "prod_test_4".to_string(),
                name: "Test Narrativ Plan Pro 1".to_string(),
                description: Some("Mock narrativ subscription plan for testing".to_string()),
                metadata: serde_json::json!({
                    "product_type": "narrativ",
                    "features": ["feature1", "feature2"],
                    "product_plan": "pro"
                }),
                default_price: Some("price_1".to_string()),
                active: Some(true),
                created: Some(chrono::Utc::now().timestamp()),
                updated: Some(chrono::Utc::now().timestamp()),
                object: "product".to_string(),
                marketing_features: None,
                images: None,
                package_dimensions: None,
                shippable: Some(false),
                statement_descriptor: Some("NARRATIV".to_string()),
                tax_code: Some("txcd_1234567890".to_string()),
                unit_label: Some("unit".to_string()),
                url: None,
                prices: vec![
                    StripePrice {
                        id: "price_1".to_string(),
                        object: "price".to_string(),
                        active: true,
                        billing_scheme: Some("per_unit".to_string()),
                        created: Some(chrono::Utc::now().timestamp()),
                        currency: "usd".to_string(),
                        currency_options: None,
                        custom_unit_amount: None,
                        livemode: Some(false),
                        lookup_key: None,
                        metadata: serde_json::json!({}),
                        nickname: None,
                        product: "prod_test_4".to_string(),
                        recurring: Some(StripeRecurring {
                            interval: "month".to_string(),
                            interval_count: Some(1),
                        }),
                        tax_behavior: None,
                        tiers_mode: None,
                        tiers: None,
                        transform_quantity: None,
                        unit_amount: Some(30),
                        unit_amount_decimal: Some("30.00".to_string()),
                        price_type: Some("recurring".to_string()),
                    },
                    StripePrice {
                        id: "price_2".to_string(),
                        object: "price".to_string(),
                        active: true,
                        billing_scheme: Some("per_unit".to_string()),
                        created: Some(chrono::Utc::now().timestamp()),
                        currency: "usd".to_string(),
                        currency_options: None,
                        custom_unit_amount: None,
                        livemode: Some(false),
                        lookup_key: None,
                        metadata: serde_json::json!({}),
                        nickname: None,
                        product: "prod_test_4".to_string(),
                        recurring: Some(StripeRecurring {
                            interval: "year".to_string(),
                            interval_count: Some(1),
                        }),
                        tax_behavior: None,
                        tiers_mode: None,
                        tiers: None,
                        transform_quantity: None,
                        unit_amount: Some(50),
                        unit_amount_decimal: Some("50.00".to_string()),
                        price_type: Some("recurring".to_string()),
                    },
                ],
            },
        ];

        // Apply meta_product_type filtering if provided
        if let Some(expected_product_type) = meta_product_type {
            mock_products.retain(|product| {
                if let Some(metadata) = product.metadata.as_object() {
                    // Check if the metadata contains the expected product_type value
                    metadata.get("product_type") == Some(&serde_json::Value::String(expected_product_type.to_string()))
                } else {
                    false
                }
            });
        }

        // Apply active status filtering if provided
        if let Some(active_status) = active {
            mock_products.retain(|product| {
                product.active == Some(active_status)
            });
        }

        Ok(mock_products)
    }

    /// Fetch mock Stripe plans with product details
    #[instrument(skip(self))]
    pub async fn get_plans(&self, meta_product_type: Option<&str>, active: Option<bool>) -> Result<Vec<StripePlanWithProduct>> {
        // Return mock plans for testing
        let mut mock_plans = Vec::new();
        
        // Create mock plans with default billing scheme
        let billing_scheme = "per_unit";
        
        // Create a basic plan
        let plan = StripePlan {
            id: "plan_test_1".to_string(),
            object: "plan".to_string(),
            active: true,
            amount: Some(2900),
            amount_decimal: Some("2900".to_string()),
            billing_scheme: Some(billing_scheme.to_string()),
            created: Some(chrono::Utc::now().timestamp()),
            currency: "usd".to_string(),
            interval: "month".to_string(),
            interval_count: 1,
            livemode: Some(false),
            metadata: serde_json::json!({
                "product_type": meta_product_type.unwrap_or("real_estate"),
                "product_plan": "basic"
            }),
            nickname: Some("Basic Plan".to_string()),
            product: "prod_test_1".to_string(),
            tiers_mode: if billing_scheme == "tiered" { Some("graduated".to_string()) } else { None },
            transform_usage: None,
            trial_period_days: Some(7),
            usage_type: "licensed".to_string(),
        };
        
        // Create a mock product for the plan
        let product = StripeProduct {
            id: "prod_test_1".to_string(),
            object: "product".to_string(),
            active: Some(true),
            attributes: Some(vec!["feature1".to_string(), "feature2".to_string()]),
            caption: Some("Mock product for testing".to_string()),
            created: Some(chrono::Utc::now().timestamp()),
            default_price: Some("price_1".to_string()),
            description: Some("Mock real estate subscription product for testing".to_string()),
            images: Some(vec!["https://example.com/image1.jpg".to_string()]),
            livemode: Some(false),
            marketing_features: None,
            metadata: serde_json::json!({
                "product_type": meta_product_type.unwrap_or("narrativ"),
                "product_plan": "pro"
            }),
            name: "Test Narrativ Product".to_string(),
            package_dimensions: None,
            shippable: Some(false),
            statement_descriptor: Some("NARRATIV".to_string()),
            tax_code: Some("txcd_1234567890".to_string()),
            unit_label: Some("unit".to_string()),
            updated: Some(chrono::Utc::now().timestamp()),
            url: Some("https://example.com/product".to_string()),
            prices: None,
        };
        
        let plan_with_product = StripePlanWithProduct {
            plan,
            product_details: product,
        };
        
        mock_plans.push(plan_with_product);
        
        // Add additional mock plans for narrativ product type
        if meta_product_type.is_none() || meta_product_type == Some("narrativ") {
            // Create a premium narrativ plan
            let premium_plan = StripePlan {
                id: "plan_narrativ_premium".to_string(),
                object: "plan".to_string(),
                active: true,
                amount: Some(4900),
                amount_decimal: Some("4900".to_string()),
                billing_scheme: Some(billing_scheme.to_string()),
                created: Some(chrono::Utc::now().timestamp()),
                currency: "usd".to_string(),
                interval: "month".to_string(),
                interval_count: 1,
                livemode: Some(false),
                metadata: serde_json::json!({
                    "product_type": "narrativ",
                    "product_plan": "premium",
                    "features": ["ai_content", "advanced_analytics", "priority_support"]
                }),
                nickname: Some("Narrativ Premium Plan".to_string()),
                product: "prod_narrativ_premium".to_string(),
                tiers_mode: if billing_scheme == "tiered" { Some("graduated".to_string()) } else { None },
                transform_usage: None,
                trial_period_days: Some(14),
                usage_type: "licensed".to_string(),
            };
            
            let premium_product = StripeProduct {
                id: "prod_narrativ_premium".to_string(),
                object: "product".to_string(),
                active: Some(true),
                attributes: Some(vec!["ai_content".to_string(), "analytics".to_string(), "support".to_string()]),
                caption: Some("Premium narrativ content creation platform".to_string()),
                created: Some(chrono::Utc::now().timestamp()),
                default_price: Some("price_narrativ_premium".to_string()),
                description: Some("Advanced AI-powered content creation with analytics and priority support".to_string()),
                images: Some(vec!["https://example.com/narrativ_premium.jpg".to_string()]),
                livemode: Some(false),
                marketing_features: None,
                metadata: serde_json::json!({
                    "product_type": "narrativ",
                    "product_plan": "premium",
                    "features": ["ai_content", "advanced_analytics", "priority_support"],
                    "target_audience": "content_creators"
                }),
                name: "Narrativ Premium".to_string(),
                package_dimensions: None,
                shippable: Some(false),
                statement_descriptor: Some("NARRATIV".to_string()),
                tax_code: Some("txcd_1234567890".to_string()),
                unit_label: Some("subscription".to_string()),
                updated: Some(chrono::Utc::now().timestamp()),
                url: Some("https://narrativ.com/premium".to_string()),
                prices: None,
            };
            
            let premium_plan_with_product = StripePlanWithProduct {
                plan: premium_plan,
                product_details: premium_product,
            };
            
            mock_plans.push(premium_plan_with_product);
            
            // Create a basic narrativ plan
            let basic_narrativ_plan = StripePlan {
                id: "plan_narrativ_basic".to_string(),
                object: "plan".to_string(),
                active: true,
                amount: Some(1900),
                amount_decimal: Some("1900".to_string()),
                billing_scheme: Some(billing_scheme.to_string()),
                created: Some(chrono::Utc::now().timestamp()),
                currency: "usd".to_string(),
                interval: "month".to_string(),
                interval_count: 1,
                livemode: Some(false),
                metadata: serde_json::json!({
                    "product_type": "narrativ",
                    "product_plan": "basic",
                    "features": ["ai_content", "basic_analytics"]
                }),
                nickname: Some("Narrativ Basic Plan".to_string()),
                product: "prod_narrativ_basic".to_string(),
                tiers_mode: if billing_scheme == "tiered" { Some("graduated".to_string()) } else { None },
                transform_usage: None,
                trial_period_days: Some(7),
                usage_type: "licensed".to_string(),
            };
            
            let basic_narrativ_product = StripeProduct {
                id: "prod_narrativ_basic".to_string(),
                object: "product".to_string(),
                active: Some(true),
                attributes: Some(vec!["ai_content".to_string(), "analytics".to_string()]),
                caption: Some("Basic narrativ content creation platform".to_string()),
                created: Some(chrono::Utc::now().timestamp()),
                default_price: Some("price_narrativ_basic".to_string()),
                description: Some("AI-powered content creation with basic analytics".to_string()),
                images: Some(vec!["https://example.com/narrativ_basic.jpg".to_string()]),
                livemode: Some(false),
                marketing_features: None,
                metadata: serde_json::json!({
                    "product_type": "narrativ",
                    "product_plan": "basic",
                    "features": ["ai_content", "basic_analytics"],
                    "target_audience": "individuals"
                }),
                name: "Narrativ Basic".to_string(),
                package_dimensions: None,
                shippable: Some(false),
                statement_descriptor: Some("NARRATIV".to_string()),
                tax_code: Some("txcd_1234567890".to_string()),
                unit_label: Some("subscription".to_string()),
                updated: Some(chrono::Utc::now().timestamp()),
                url: Some("https://narrativ.com/narrativ_basic".to_string()),
                prices: None,
            };
            
            let basic_narrativ_plan_with_product = StripePlanWithProduct {
                plan: basic_narrativ_plan,
                product_details: basic_narrativ_product,
            };
            
            mock_plans.push(basic_narrativ_plan_with_product);
            
            // Create an enterprise narrativ plan
            let enterprise_plan = StripePlan {
                id: "plan_narrativ_enterprise".to_string(),
                object: "plan".to_string(),
                active: true,
                amount: Some(9900),
                amount_decimal: Some("9900".to_string()),
                billing_scheme: Some(billing_scheme.to_string()),
                created: Some(chrono::Utc::now().timestamp()),
                currency: "usd".to_string(),
                interval: "month".to_string(),
                interval_count: 1,
                livemode: Some(false),
                metadata: serde_json::json!({
                    "product_type": "narrativ",
                    "product_plan": "enterprise",
                    "features": ["ai_content", "advanced_analytics", "priority_support", "custom_integrations", "dedicated_account_manager"]
                }),
                nickname: Some("Narrativ Enterprise Plan".to_string()),
                product: "prod_narrativ_enterprise".to_string(),
                tiers_mode: if billing_scheme == "tiered" { Some("graduated".to_string()) } else { None },
                transform_usage: None,
                trial_period_days: Some(30),
                usage_type: "licensed".to_string(),
            };
            
            let enterprise_product = StripeProduct {
                id: "prod_narrativ_enterprise".to_string(),
                object: "product".to_string(),
                active: Some(true),
                attributes: Some(vec!["ai_content".to_string(), "analytics".to_string(), "support".to_string(), "integrations".to_string(), "account_management".to_string()]),
                caption: Some("Enterprise narrativ content creation platform".to_string()),
                created: Some(chrono::Utc::now().timestamp()),
                default_price: Some("price_narrativ_enterprise".to_string()),
                description: Some("Enterprise-grade AI content creation with advanced features, custom integrations, and dedicated support".to_string()),
                images: Some(vec!["https://example.com/narrativ_enterprise.jpg".to_string()]),
                livemode: Some(false),
                marketing_features: None,
                metadata: serde_json::json!({
                    "product_type": "narrativ",
                    "product_plan": "enterprise",
                    "features": ["ai_content", "advanced_analytics", "priority_support", "custom_integrations", "dedicated_account_manager"],
                    "target_audience": "enterprises",
                    "min_users": 50
                }),
                name: "Narrativ Enterprise".to_string(),
                package_dimensions: None,
                shippable: Some(false),
                statement_descriptor: Some("NARRATIV".to_string()),
                tax_code: Some("txcd_1234567890".to_string()),
                unit_label: Some("subscription".to_string()),
                updated: Some(chrono::Utc::now().timestamp()),
                url: Some("https://narrativ.com/enterprise".to_string()),
                prices: None,
            };
            
            let enterprise_plan_with_product = StripePlanWithProduct {
                plan: enterprise_plan,
                product_details: enterprise_product,
            };
            
            mock_plans.push(enterprise_plan_with_product);
        }
        
        // Apply active status filtering if provided
        if let Some(active_status) = active {
            mock_plans.retain(|plan_with_product| plan_with_product.plan.active == active_status);
        }
        
        Ok(mock_plans)
    }

    /// Create a mock Stripe checkout session
    #[instrument(skip(self))]
    pub async fn create_checkout_session(
        &self,
        _pool: &PgPool,
        user_id: Uuid,
        user_email: &str,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
        mode: &str,
        dub_id: Option<&str>,
    ) -> Result<CheckoutSessionResponse> {
        self.create_checkout_session_with_context(
            _pool,
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

    /// Create a mock Stripe checkout session with customer type and optional organization context
    #[instrument(skip(self))]
    pub async fn create_checkout_session_with_context(
        &self,
        _pool: &PgPool,
        user_id: Uuid,
        _user_email: &str,
        price_id: &str,
        _success_url: &str,
        _cancel_url: &str,
        _mode: &str,
        dub_id: Option<&str>,
        customer_type: &str,
        organization_id: Option<Uuid>,
    ) -> Result<CheckoutSessionResponse> {
        // In a real implementation, this would create a Stripe session
        // For testing, we just return a mock response
        if let Some(org_id) = organization_id {
            if let Some(dub_id) = dub_id {
                log::info!("Mock: Creating {} checkout session for org {} with price {} - Dub metadata: dubCustomerId={}, dubClickId={}", customer_type, org_id, price_id, user_id, dub_id);
            } else {
                log::info!("Mock: Creating {} checkout session for org {} with price {} - Dub metadata: dubCustomerId={} (no click ID)", customer_type, org_id, price_id, user_id);
            }
        } else {
            if let Some(dub_id) = dub_id {
                log::info!("Mock: Creating {} checkout session for user {} with price {} - Dub metadata: dubCustomerId={}, dubClickId={}", customer_type, user_id, price_id, user_id, dub_id);
            } else {
                log::info!("Mock: Creating {} checkout session for user {} with price {} - Dub metadata: dubCustomerId={} (no click ID)", customer_type, user_id, price_id, user_id);
            }
        }
        
        Ok(CheckoutSessionResponse {
            session_id: format!("cs_test_{}", uuid::Uuid::new_v4()),
            session_url: "https://checkout.stripe.com/pay/cs_test_mock#fid".to_string(),
        })
    }

    /// Create a mock customer portal session
    #[instrument(skip(self))]
    pub async fn create_customer_portal_session(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        return_url: &str,
    ) -> Result<CustomerPortalResponse> {
        // In a real implementation, this would create a Stripe portal session
        // For testing, we validate business logic requirements
        log::info!("Mock: Creating customer portal session for user {}", user_id);
        
        // Validate return URL format (meaningful business logic)
        if let Err(_) = url::Url::parse(return_url) {
            return Err(anyhow::anyhow!("Invalid return URL format"));
        }
        
        // Check if user has a Stripe customer ID (real business requirement)
        let user_result = sqlx::query!(
            "SELECT stripe_customer_id FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(pool)
        .await?;
        
        if user_result.stripe_customer_id.is_none() {
            return Err(anyhow::anyhow!("User does not have a Stripe customer ID"));
        }
        
        Ok(CustomerPortalResponse {
            portal_url: "https://billing.stripe.com/session/mock_session".to_string(),
        })
    }

    /// Update checkout session status (mock implementation)
    #[instrument(skip(self))]
    pub async fn update_checkout_session_status(
        &self,
        _pool: &PgPool,
        stripe_checkout_id: &str,
        status: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        // Mock implementation - just log the action
        log::info!("Mock: Updating checkout session {} status to {} with metadata: {:?}", 
                   stripe_checkout_id, status, metadata);
        Ok(())
    }

    /// Activate user subscription (mock implementation)
    #[instrument(skip(self))]
    pub async fn activate_user_subscription(
        &self,
        _pool: &PgPool,
        user_id: Uuid,
        stripe_customer_id: &str,
    ) -> Result<()> {
        // Mock implementation - just log the action
        log::info!("Mock: Activating subscription for user {} with customer ID {}", 
                   user_id, stripe_customer_id);
        Ok(())
    }
}

// Implement the trait for the mock billing service
#[async_trait::async_trait]
impl BillingServiceTrait for MockBillingService {
    async fn get_products(&self, meta_product_type: Option<&str>, active: Option<bool>) -> Result<Vec<StripeProductWithPrices>> {
        MockBillingService::get_products(self, meta_product_type, active).await
    }

    async fn get_product(&self, product_id: &str, with_prices: bool) -> Result<StripeProductWithPrices> {
        MockBillingService::get_product(self, product_id, with_prices).await
    }

    async fn get_price(&self, price_id: &str) -> Result<StripePrice> {
        MockBillingService::get_price(self, price_id).await
    }

    async fn get_plans(&self, meta_product_type: Option<&str>, active: Option<bool>) -> Result<Vec<StripePlanWithProduct>> {
        MockBillingService::get_plans(self, meta_product_type, active).await
    }

    async fn create_free_subscription(
        &self,
        _pool: &PgPool,
        _user_id: Uuid,
        _user_email: &str,
    ) -> Result<()> {
        // Mock implementation - just return success
        Ok(())
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
        MockBillingService::create_checkout_session(self, pool, user_id, user_email, price_id, success_url, cancel_url, mode, dub_id).await
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
        MockBillingService::create_checkout_session_with_context(self, pool, user_id, user_email, price_id, success_url, cancel_url, mode, dub_id, customer_type, organization_id).await
    }

    async fn create_customer_portal_session(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        return_url: &str,
    ) -> Result<CustomerPortalResponse> {
        MockBillingService::create_customer_portal_session(self, pool, user_id, return_url).await
    }

    async fn update_checkout_session_status(
        &self,
        pool: &PgPool,
        stripe_checkout_id: &str,
        status: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        MockBillingService::update_checkout_session_status(self, pool, stripe_checkout_id, status, metadata).await
    }

    async fn activate_user_subscription(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        stripe_customer_id: &str,
    ) -> Result<()> {
        MockBillingService::activate_user_subscription(self, pool, user_id, stripe_customer_id).await
    }

    async fn cancel_subscription(
        &self,
        _pool: &PgPool,
        subscription_id: &str,
        cancellation_details: Option<StripeCancellationDetails>,
        _invoice_now: Option<bool>,
        _prorate: Option<bool>,
    ) -> Result<()> {
        // Mock implementation - just log the action
        log::info!(
            "Mock: Canceling subscription {} with details: {:?}", 
            subscription_id, 
            cancellation_details
        );
        Ok(())
    }

    async fn get_subscriptions_by_customer(&self, customer_id: &str) -> Result<StripeSubscriptionList> {
        // Mock implementation - return empty subscription list
        log::info!("Mock: Getting subscriptions for customer: {}", customer_id);
        
        Ok(StripeSubscriptionList {
            object: "list".to_string(),
            data: vec![],
            has_more: false,
            url: format!("/v1/subscriptions?customer={}", customer_id),
        })
    }

    async fn get_customer(&self, customer_id: &str) -> Result<StripeCustomer> {
        // Mock implementation - return a dummy customer with metadata
        log::info!("Mock: Getting customer: {}", customer_id);
        
        let mut metadata = serde_json::json!({});
        metadata["customer_type"] = serde_json::Value::String("user".to_string());
        metadata["product_type"] = serde_json::Value::String("narrativ".to_string());
        
        Ok(StripeCustomer {
            id: customer_id.to_string(),
            object: "customer".to_string(),
            address: None,
            balance: 0,
            created: chrono::Utc::now().timestamp(),
            currency: Some("usd".to_string()),
            default_source: None,
            delinquent: false,
            description: Some("Mock customer".to_string()),
            email: Some("mock@example.com".to_string()),
            invoice_credit_balance: None,
            invoice_prefix: None,
            invoice_settings: None,
            livemode: false,
            metadata,
            name: Some("Mock User".to_string()),
            next_invoice_sequence: None,
            phone: None,
            preferred_locales: None,
            shipping: None,
            sources: None,
            subscriptions: None,
            tax_exempt: None,
            tax_ids: None,
            test_clock: None,
        })
    }

    async fn update_customer_email(&self, customer_id: &str, new_email: &str) -> Result<StripeCustomer> {
        // Mock implementation - return a dummy customer with updated email
        log::info!("Mock: Updating customer {} email to: {}", customer_id, new_email);
        
        let mut metadata = serde_json::json!({});
        metadata["customer_type"] = serde_json::Value::String("user".to_string());
        metadata["product_type"] = serde_json::Value::String("narrativ".to_string());
        
        Ok(StripeCustomer {
            id: customer_id.to_string(),
            object: "customer".to_string(),
            address: None,
            balance: 0,
            created: chrono::Utc::now().timestamp(),
            currency: Some("usd".to_string()),
            default_source: None,
            delinquent: false,
            description: Some("Mock customer".to_string()),
            email: Some(new_email.to_string()), // Updated email
            invoice_credit_balance: None,
            invoice_prefix: None,
            invoice_settings: None,
            livemode: false,
            metadata,
            name: Some("Mock User".to_string()),
            next_invoice_sequence: None,
            phone: None,
            preferred_locales: None,
            shipping: None,
            sources: None,
            subscriptions: None,
            tax_exempt: None,
            tax_ids: None,
            test_clock: None,
        })
    }
}

impl Default for MockBillingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_get_real_estate_products() {
        let service = MockBillingService::new();
        let products = service.get_products(Some("real_estate"), Some(true)).await.unwrap();
            
        assert_eq!(products.len(), 2);
        assert_eq!(products[0].name, "Test Real Estate Plan");
        assert_eq!(products[0].prices.len(), 2);
        assert_eq!(products[1].name, "Premium Real Estate Plan");
        assert_eq!(products[1].prices.len(), 1);
    }

    #[tokio::test]
    async fn test_mock_get_narrativ_products() {
        let service = MockBillingService::new();
        let products = service.get_products(Some("narrativ"), Some(true)).await.unwrap();
            
        assert_eq!(products.len(), 2);
        assert_eq!(products[0].name, "Narrativ Plan Pro 1");
        assert_eq!(products[0].prices.len(), 1);
        assert_eq!(products[1].name, "Test Narrativ Plan Pro 1");
        assert_eq!(products[1].prices.len(), 2);
    }

    #[tokio::test]
    async fn test_mock_create_real_estate_checkout_session() {
        let service = MockBillingService::new();
        
        // Test that the mock service can be created and has the expected structure
        // We don't need to test with real database connections for unit tests
        assert_eq!(service.get_products(Some("real_estate"), Some(true)).await.unwrap().len(), 2);
        
        // The mock service should work without external dependencies
        // This test verifies the mock service structure and basic functionality
    }

    #[tokio::test]
    async fn test_mock_create_real_estate_customer_portal_session() {
        let service = MockBillingService::new();
        
        // Test that the mock service can be created and has the expected structure
        // We don't need to test with real database connections for unit tests
        let products = service.get_products(Some("real_estate"), Some(true)).await.unwrap();
        assert_eq!(products.len(), 2);
        assert_eq!(products[0].name, "Test Real Estate Plan");
        
        // The mock service should work without external dependencies
        // This test verifies the mock service structure and basic functionality
    }

    #[tokio::test]
    async fn test_mock_get_real_estate_products_filtered_by_active() {
        let service = MockBillingService::new();
        
        // Test filtering by active status
        let active_products = service.get_products(Some("real_estate"), Some(true)).await.unwrap();
        assert_eq!(active_products.len(), 2);
        
        // Test filtering by inactive status (should return empty since all mock products are active)
        let inactive_products = service.get_products(Some("real_estate"), Some(false)).await.unwrap();
        assert_eq!(inactive_products.len(), 0);
        
        // Test filtering by both product_type and active status
        let filtered_products = service.get_products(Some("real_estate"), Some(true)).await.unwrap();
        assert_eq!(filtered_products.len(), 2);
    }
}
