use crate::{
    db::{Db, Payment},
    electrum::rpc_async,
    utils::{encrypt_wif, new_key, script_hash},
};
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use std::env;
use tokio::task::spawn_blocking;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
struct PayReq {
    amount: f64,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/payments").route(web::post().to(create_payment)))
        .service(web::resource("/payments/{id}").route(web::get().to(get_payment)));
}

#[instrument(skip(db))]
async fn create_payment(db: web::Data<Db>, req: web::Json<PayReq>) -> HttpResponse {
    if req.amount <= 0.0 {
        debug!(
            "Rejected payment request with invalid amount: {}",
            req.amount
        );
        return HttpResponse::BadRequest().finish();
    }

    let id = Uuid::new_v4().to_string();
    let (_, wif, addr) = new_key();
    let wif_enc = encrypt_wif(&wif);

    let payment = Payment {
        id: id.clone(),
        address: addr.clone(),
        wif_enc,
        amount: req.amount,
        status: "pending".into(),
        created_at: 0,
        updated_at: 0,
    };

    // DB insert offloaded to blocking thread
    let db_clone = db.clone();
    let payment_clone = payment.clone();
    if let Err(e) = spawn_blocking(move || db_clone.insert(&payment_clone))
        .await
        .expect("blocking task panicked")
    {
        error!("DB insert error: {}", e);
        return HttpResponse::InternalServerError().finish();
    }

    info!("Created new payment: id={}, address={}", id, addr);
    HttpResponse::Ok().json(json!({ "id": id, "address": addr, "amount": req.amount }))
}

#[instrument(skip(db))]
async fn get_payment(db: web::Data<Db>, path: web::Path<String>) -> HttpResponse {
    let payment_id = path.into_inner();
    debug!("Looking up payment with id: {}", payment_id);

    // DB lookup offloaded to blocking thread
    let db_clone = db.clone();
    let payment_id_clone = payment_id.clone();
    let payment_res = spawn_blocking(move || db_clone.find(&payment_id_clone))
        .await
        .expect("blocking task panicked");

    let payment_opt = match payment_res {
        Ok(p) => p,
        Err(e) => {
            error!("DB lookup error: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let Some(payment) = payment_opt else {
        debug!("Payment not found: {}", payment_id);
        return HttpResponse::NotFound().finish();
    };

    debug!(
        "Fetching blockchain information for address: {}",
        payment.address
    );

    let bal = match rpc_async(
        "blockchain.scripthash.get_balance",
        &[script_hash(&payment.address).into()],
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            error!("Electrum error getting balance: {:?}", e);
            return HttpResponse::BadGateway().body("electrum error");
        }
    };

    let hdr = match rpc_async("blockchain.headers.subscribe", &[]).await {
        Ok(v) => v,
        Err(e) => {
            error!("Electrum error subscribing to headers: {:?}", e);
            return HttpResponse::BadGateway().body("electrum error");
        }
    };

    let hist = match rpc_async(
        "blockchain.scripthash.get_history",
        &[script_hash(&payment.address).into()],
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            error!("Electrum error getting history: {:?}", e);
            return HttpResponse::BadGateway().body("electrum error");
        }
    };

    let tip = hdr["height"].as_u64().unwrap_or(0);
    let confirmations = hist
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["height"].as_u64().unwrap_or(0) > 0)
        .map(|h| tip - h["height"].as_u64().unwrap() + 1)
        .min()
        .unwrap_or(0);

    // Show total received (confirmed + unconfirmed) so user sees pending funds
    let confirmed_sat = bal["confirmed"].as_i64().unwrap_or(0).max(0) as f64;
    let unconfirmed_sat = bal["unconfirmed"].as_i64().unwrap_or(0).max(0) as f64;
    let received = (confirmed_sat + unconfirmed_sat) / 1e8;

    let confirmations_needed = env::var("CONFIRMATIONS")
        .unwrap_or_else(|_| "2".to_string())
        .parse::<u64>()
        .unwrap_or(2);

    info!(
        "Payment info: id={}, address={}, confirmations={}/{}",
        payment.id, payment.address, confirmations, confirmations_needed
    );

    HttpResponse::Ok().json(json!({
        "id": payment.id,
        "address": payment.address,
        "amount": payment.amount,
        "status": payment.status,
        "created_at": payment.created_at,
        "updated_at": payment.updated_at,
        "confirmations": confirmations,
        "confirmations_needed": confirmations_needed,
        "received": received,
    }))
}
