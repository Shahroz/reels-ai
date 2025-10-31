use anyhow::Result;
use stripe::{Client, Charge, CreateCharge, Currency, PaymentIntent, CreatePaymentIntent};
use std::env;
use tracing::instrument;

pub struct StripePayment {
    client: Client,
}

impl StripePayment {
    pub fn new() -> Self {
        let secret = env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY must be set");
        let client = Client::new(secret);
        StripePayment { client }
    }

    #[instrument(skip(self, amount, currency))]
    pub async fn create_payment_intent(&self, amount: i64, currency: Currency) -> Result<PaymentIntent> {
        let mut params = stripe::CreatePaymentIntent::new(amount, currency);
        // Additional parameter configuration as needed
        let payment_intent = PaymentIntent::create(&self.client, params, None).await?;
        Ok(payment_intent)
    }

    #[instrument(skip(self, customer_id, amount))]
    pub async fn charge_customer(&self, customer_id: &str, amount: i64) -> Result<Charge> {
        let mut params = stripe::CreateCharge::new(amount, Currency::USD);
        params.customer = Some(customer_id.to_string());
        let charge = Charge::create(&self.client, params, None).await?;
        Ok(charge)
    }

    #[instrument(skip(self, email))]
    pub async fn create_stripe_customer(&self, email: &str) -> Result<String> {
        let customer = stripe::Customer::create(&self.client, stripe::CreateCustomer {
            email: Some(email),
            description: Some("New customer for StyleClone".into()),
            ..Default::default()
        }, None).await?;
        Ok(customer.id)
    }
}

#[cfg(FALSE)]
mod tests {
    use super::*;
    use stripe::Currency;

    #[tokio::test]
    async fn test_create_stripe_customer() {
        let stripe_payment = StripePayment::new();
        // In testing, use a known test email.
        let result = stripe_payment.create_stripe_customer("test@example.com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_payment_intent() {
        let stripe_payment = StripePayment::new();
        let result = stripe_payment.create_payment_intent(5000, Currency::USD).await;
        assert!(result.is_ok());
    }
}
