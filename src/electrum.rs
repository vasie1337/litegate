use anyhow::{anyhow, bail, Context, Result};
use electrum_client::{Client, ConfigBuilder, ElectrumApi, Param};
use lazy_static::lazy_static;
use serde_json::Value;
use std::{sync::Mutex, thread, time::Duration};
use tracing::{debug, error, info, trace};

/// Open a single connection (`scheme://host:port`) and perform `server.version`.
fn connect_once(host: &str, port: &str, scheme: &str) -> Result<Client> {
    let url = format!("{scheme}://{host}:{port}");
    debug!(url = %url, "connecting");

    let cfg = ConfigBuilder::new()
        .timeout(Some(15))
        .retry(0) // manual retry below
        .validate_domain(false)
        .build();

    let c = Client::from_config(&url, cfg).with_context(|| format!("dial {url}"))?;

    debug!(url = %url, "handshaking");
    c.raw_call(
        "server.version",
        vec![
            Param::String("ltc-payments/1.0".into()),
            Param::String("1.4".into()),
        ],
    )
    .with_context(|| format!("handshake {url}"))?;

    info!(url = %url, "electrum ready");
    Ok(c)
}

/// Return a live client connected to the working Electrum server.
fn fresh_client() -> Result<Client> {
    // Always use the hardcoded working configuration - TCP on port 50001
    let host = "electrum.ltc.xurious.com";
    let port = "50001";
    let scheme = "tcp";

    debug!(scheme = %scheme, host = %host, port = %port, "Using fixed connection");

    match connect_once(host, port, scheme) {
        Ok(c) => Ok(c),
        Err(e) => {
            error!(scheme = %scheme, host = %host, port = %port, error = ?e, "connection failed");
            Err(anyhow!("connection to {scheme}://{host}:{port} failed").context(e))
        }
    }
}

lazy_static! {
    static ref ECL: Mutex<Option<Client>> = Mutex::new(None);
}

fn client() -> Result<&'static Mutex<Option<Client>>> {
    if ECL.lock().unwrap().is_none() {
        *ECL.lock().unwrap() = Some(fresh_client()?);
    }
    Ok(&ECL)
}

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

/// Synchronous RPC wrapper with verbose logs + automatic reconnect.
pub fn rpc(method: &str, params: &[Value]) -> Result<Value> {
    for attempt in 1..=3 {
        let lock = client()?;
        let mut guard = lock.lock().unwrap();

        if guard.is_none() {
            *guard = Some(fresh_client()?);
        }

        trace!(method = %method, params = ?params, attempt = attempt, "rpc attempt");
        match guard.as_mut().unwrap().raw_call(method, to_params(params)) {
            Ok(v) => {
                debug!(method = %method, "rpc success");
                return Ok(v);
            }
            Err(e) => {
                error!(method = %method, attempt = attempt, error = ?e, "rpc failed");
                *guard = None; // drop broken connection
                drop(guard);
                if attempt < 3 {
                    thread::sleep(Duration::from_millis(500 * attempt as u64));
                }
            }
        }
    }
    bail!("rpc fail after 3 attempts (method='{method}')")
}

/// Synchronous vsize × sat/vB via `blockchain.estimatefee`.
pub fn fee_sat(vsize: u64) -> u64 {
    let est = rpc("blockchain.estimatefee", &[Value::from(6)])
        .unwrap_or(Value::from(0.0))
        .as_f64()
        .unwrap_or(0.0);

    let sat_per_vb = ((est * 1e8) / 1000.0).ceil() as u64;
    vsize * sat_per_vb.max(1)
}

/// **Async** wrapper around `rpc`, executed in a dedicated blocking thread.
pub async fn rpc_async(method: &str, params: &[Value]) -> Result<Value> {
    let method_owned = method.to_owned();
    let params_vec: Vec<Value> = params.iter().cloned().collect();
    tokio::task::spawn_blocking(move || rpc(&method_owned, &params_vec))
        .await
        .map_err(|e| anyhow!("join error: {}", e))?
}

/// **Async** wrapper around `fee_sat`, executed in a blocking thread.
pub async fn fee_sat_async(vsize: u64) -> u64 {
    tokio::task::spawn_blocking(move || fee_sat(vsize))
        .await
        .unwrap_or(vsize) // fall back to vsize × 1 sat/vB on join error
}
