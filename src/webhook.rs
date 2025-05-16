use crate::db::Payment;
use anyhow::{anyhow, Context, Result};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde_json::json;
use sha2::Sha256;
use std::env;
use tracing::{debug, error, info};

type HmacSha256 = Hmac<Sha256>;

pub async fn send_completion_webhook(payment: &Payment) -> Result<()> {
    let webhook_url = env::var("WEBHOOK_URL").context("WEBHOOK_URL env missing")?;
    if webhook_url.is_empty() {
        debug!("WEBHOOK_URL is empty, skipping webhook");
        return Ok(());
    }

    let webhook_secret = env::var("WEBHOOK_SECRET").context("WEBHOOK_SECRET env missing")?;
    
    let payload = json!({
        "event": "payment.completed",
        "payment": {
            "id": payment.id,
            "address": payment.address,
            "amount": payment.amount,
            "status": payment.status,
            "created_at": payment.created_at,
            "updated_at": payment.updated_at,
            "expires_at": payment.expires_at,
        }
    });
    
    let payload_str = payload.to_string();
    
    let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes())
        .map_err(|_| anyhow!("Invalid webhook secret length"))?;
    mac.update(payload_str.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    
    info!(payment_id = %payment.id, "Sending webhook for completed payment");
    
    let client = Client::new();
    let response = client
        .post(&webhook_url)
        .header("Content-Type", "application/json")
        .header("X-Signature", &signature)
        .body(payload_str)
        .send()
        .await
        .context("Failed to send webhook request")?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!(payment_id = %payment.id, %status, %body, "Webhook failed");
        return Err(anyhow!("Webhook failed with status: {}", status));
    }
    
    info!(payment_id = %payment.id, "Webhook sent successfully");
    Ok(())
}