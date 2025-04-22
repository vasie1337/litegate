use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;
use crate::{
    db::{Db, Payment},
    utils::{new_key, encrypt_wif, script_hash},
    electrum::rpc,
};
use serde_json::json;

#[derive(Deserialize)]
struct PayReq { amount: f64 }

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/payments").route(web::post().to(create_payment)))
       .service(web::resource("/payments/{id}").route(web::get().to(get_payment)));
}

async fn create_payment(db: web::Data<Db>, req: web::Json<PayReq>) -> HttpResponse {
    if req.amount <= 0.0 { return HttpResponse::BadRequest().finish() }
    let id = Uuid::new_v4().to_string();
    let (_, wif, addr) = new_key();
    let wif_enc = encrypt_wif(&wif);
    db.insert(&Payment {
        id: id.clone(),
        address: addr.clone(),
        wif_enc,
        amount: req.amount,
        status: "pending".into(),
        created_at: 0,
        updated_at: 0,
    });
    HttpResponse::Ok().json(json!({ "id": id, "address": addr, "amount": req.amount }))
}

async fn get_payment(db: web::Data<Db>, path: web::Path<String>) -> HttpResponse {
    let Some(p) = db.find(&path) else { return HttpResponse::NotFound().finish() };

    let bal = match rpc("blockchain.scripthash.get_balance", &[script_hash(&p.address).into()]) {
        Ok(v) => v,
        Err(_) => return HttpResponse::BadGateway().body("electrum error"),
    };
    let hdr = match rpc("blockchain.headers.subscribe", &[]) {
        Ok(v) => v,
        Err(_) => return HttpResponse::BadGateway().body("electrum error"),
    };
    let hist = match rpc("blockchain.scripthash.get_history", &[script_hash(&p.address).into()]) {
        Ok(v) => v,
        Err(_) => return HttpResponse::BadGateway().body("electrum error"),
    };

    let tip = hdr["height"].as_u64().unwrap();
    let confirmations = hist.as_array().unwrap()
        .iter()
        .filter(|h| h["height"].as_u64().unwrap() > 0)
        .map(|h| tip - h["height"].as_u64().unwrap() + 1)
        .min()
        .unwrap_or(0);

    let received   = bal["confirmed"].as_i64().unwrap_or(0).max(0) as f64 / 1e8;

    HttpResponse::Ok().json(json!({
        "id": p.id,
        "address": p.address,
        "amount": p.amount,
        "status": p.status,
        "created_at": p.created_at,
        "updated_at": p.updated_at,
        "confirmations": confirmations,
        "confirmations_needed": std::env::var("CONFIRMATIONS_NEEDED").unwrap_or_else(|_| "2".to_string()).parse::<u64>().unwrap_or(2),
        "received": received,
    }))
}
