use anyhow::{anyhow, bail, Context, Result};
use electrum_client::{Client, ConfigBuilder, ElectrumApi, Param};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::thread;
use std::time::Duration;
use tokio::sync::Mutex as AsyncMutex;
use tracing::{debug, error, info, trace};

fn connect_once(host: &str, port: &str, scheme: &str) -> Result<Client> {
    let url = format!("{scheme}://{host}:{port}");
    debug!(%url, "connecting");

    let cfg = ConfigBuilder::new()
        .timeout(Some(15))
        .retry(0)
        .validate_domain(false)
        .build();

    let c = Client::from_config(&url, cfg).with_context(|| format!("dial {url}"))?;
    debug!(%url, "handshaking");
    c.raw_call(
        "server.version",
        vec![
            Param::String("ltc-payments/1.0".into()),
            Param::String("1.4".into()),
        ],
    )
    .with_context(|| format!("handshake {url}"))?;

    info!(%url, "electrum ready");
    Ok(c)
}

fn fresh_client() -> Result<Client> {
    connect_once("electrum.ltc.xurious.com", "50001", "tcp")
        .map_err(|e| anyhow!("connection failed").context(e))
}

static POOL: Lazy<AsyncMutex<Vec<Client>>> = Lazy::new(|| AsyncMutex::new(Vec::new()));

fn json_param(v: &Value) -> Param {
    match v {
        Value::String(s) => Param::String(s.clone()),
        Value::Bool(b) => Param::Bool(*b),
        Value::Number(n) => n
            .as_u64()
            .map(|u| {
                if u <= u32::MAX as u64 {
                    Param::U32(u as u32)
                } else {
                    Param::Usize(u as usize)
                }
            })
            .unwrap_or_else(|| Param::String(n.to_string())),
        Value::Array(arr) => {
            Param::Bytes(arr.iter().map(|x| x.as_u64().unwrap_or(0) as u8).collect())
        }
        _ => Param::String(v.to_string()),
    }
}

fn to_params(v: &[Value]) -> Vec<Param> {
    v.iter().map(json_param).collect()
}

/// synchronous RPC; pulls a client from the pool, performs the call, returns it
fn rpc_sync(method: &str, params: &[Value]) -> Result<Value> {
    for attempt in 1..=3 {
        let mut guard = POOL.blocking_lock();
        let client = match guard.pop() {
            Some(c) => c,
            None => fresh_client()?,
        };
        drop(guard);

        trace!(%method, ?params, attempt, "rpc attempt");
        match client.raw_call(method, to_params(params)) {
            Ok(v) => {
                debug!(%method, "rpc success");
                POOL.blocking_lock().push(client);
                return Ok(v);
            }
            Err(e) => {
                error!(%method, attempt, ?e, "rpc failed");
                if attempt < 3 {
                    thread::sleep(Duration::from_millis(500 * attempt as u64));
                }
            }
        }
    }
    bail!("rpc fail after 3 attempts ({method})")
}

pub async fn rpc_async(method: &str, params: &[Value]) -> Result<Value> {
    let m = method.to_owned();
    let p: Vec<Value> = params.iter().cloned().collect();
    tokio::task::spawn_blocking(move || rpc_sync(&m, &p))
        .await
        .map_err(|e| anyhow!("join error {e}"))?
}

pub fn fee_sat(vsize: u64) -> u64 {
    let est = rpc_sync("blockchain.estimatefee", &[Value::from(6)])
        .unwrap_or(Value::from(0.0))
        .as_f64()
        .unwrap_or(0.0);
    let sat_per_vb = ((est * 1e8) / 1000.0).ceil() as u64;
    vsize * sat_per_vb.max(1)
}

pub async fn fee_sat_async(vsize: u64) -> u64 {
    tokio::task::spawn_blocking(move || fee_sat(vsize))
        .await
        .unwrap_or(vsize)
}
