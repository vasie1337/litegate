
# LTC Payment Gateway

A minimal Rust + Actix-Web service that issues generated single-use Litecoin (LTC) deposit addresses, tracks incoming payments through an Electrum server, and automatically sweeps confirmed funds into a cold wallet.

## 1 • High-level Architecture

```
┌────────────┐   HTTP/JSON   ┌──────────────┐
│ Front-end  │ ────────────▶ │  Actix-Web   │
└────────────┘               │  routes.rs   │
                             ├──────────────┤
                             │   db.rs      │ ← SQLite (payments.db)
                             ├──────────────┤
 Electrum  ⇆ electrum.rs ⇆   │ sweeper.rs   │ ⇆ Litecoin network
 server                      └──────────────┘
```

* **/src/routes.rs** – small REST surface (`POST /payments`, `GET /payments/{id}`)  
* **db.rs** – SQLite wrapper (table **payments**)  
* **electrum.rs** – thin Electrum RPC pool (no full node needed)  
* **sweeper.rs** – background worker that “ticks” every 10 s, detects confirmed funds and constructs a sweeping transaction  
* **utils.rs** – key-gen, Bech32 address helpers, AES-GCM encryption for the private key (WIF)

---

## 2 • Environment

Variable | Purpose
---------|---------
`MAIN_ADDRESS` | Cold wallet the sweeper pays to  
`AES_KEY` | 32-byte hex key for AES-GCM WIF encryption  
`ELECTRUM_HOST / PORT` | Upstream Electrum daemon  
`CONFIRMATIONS` | Blocks required before sweeping (default 2)  
`DB_FILE` | SQLite path (default `payments.db`)  
`PORT` | HTTP port (default 3000)

Copy `.env.sample`, fill in real values, then:

```bash
cargo run --release
```

---

## 3 • API Flows

### 3.1 Happy path

| Step | Request (curl) | Typical Response |
|------|----------------|------------------|
| ① Create payment | `POST /payments`<br>`{ "amount": 0.5, "ttl": 900 }` | `{ "id": "...", "address": "ltc1...", "amount": 0.5, "expires_at": 1713875023 }` |
| ② User sends 0.5 LTC | On-chain | — |
| ③ Poll status | `GET /payments/{id}` | `{ "status":"pending", "confirmations":1, "received":0.5 }` |
| ④ ≥ 2 confs reached | automatic | record in DB marked **completed** |
| ⑤ Sweep | sweeper builds a tx → broadcasts → funds arrive in `MAIN_ADDRESS` |

### 3.2 Expired / unpaid

* TTL > 0 puts a hard deadline (`expires_at`).  
* On poll, server auto-marks as **expired** if now > `expires_at` and still zero confs.  
* Sweeper ignores expired invoices.

### 3.3 Under- / Over-payment

* **Under-payment**  
  * `sweeper.rs` requires `confirmed_balance ≥ amount` (see `sweep_threshold`).  
  * If less, the invoice stays **pending** and will eventually flip to **expired** after `ttl`.  
  * Nothing is swept; payer must top-up to reach the requested amount.

* **Exact / Over-payment**  
  * As soon as the confirmed balance meets or exceeds the requested `amount`, the sweeper broadcasts a tx.  
  * **All** coins on the deposit address (over-payment included) are forwarded to `MAIN_ADDRESS`.


---

## 4 • Internal Tick System

* A single Tokio task (`sweeper::start`) runs forever.  
* **Interval**: 10 s (`interval(Duration::from_secs(10))`).  
* **cycle** counter increments each tick.  
* For every payment row:  
  * **Hot entries** (`status == "pending"`) are processed **every tick**.  
  * **Cold entries** (any other status) are processed once per **360 ticks ≈ 1 h** to finalise edge cases or confirm sweeps.

```text
tick = 0,10,20,…         // 10 s cadence
cycle%360==0  ──▶ cold scan
else            ──▶ hot scan only
```

This keeps pending invoices very responsive while preventing useless RPC spam for already-handled ones.

---

## 5 • Payment States

State | Meaning | Transition
------|---------|-----------
`pending` | Address issued, waiting for funds | → `expired` (TTL up) / `completed` (swept)
`expired` | TTL passed with < needed confirmations | terminal
`completed` | Funds swept to cold wallet | terminal

---

## 6 • Database Schema

```sql
CREATE TABLE payments(
  id TEXT PRIMARY KEY,
  address TEXT UNIQUE,
  wif_enc TEXT NOT NULL,
  amount REAL,
  status TEXT,            -- pending/expired/completed
  created_at INTEGER,     -- set by trigger in INSERT
  updated_at INTEGER,     -- AUTOINC on updates
  expires_at INTEGER
);
CREATE INDEX idx_payments_expires_at ON payments(expires_at);
```

(All timestamps are Unix seconds.)

---

## 7 • Security Notes

* Private key (WIF) only ever touches disk encrypted (AES-256-GCM).  
* Sweeper decrypts the WIF in-memory just long enough to sign the sweep.  
* No incoming ports; all chain data fetched via Electrum over TCP/TLS.  

---

## 8 • What to Expect

* **10-second latency** on invoice updates; **≈ 1–3 min** until sweep after required confirmations.  
* If Electrum is down the gateway continues issuing addresses; sweeper resumes when connectivity is back.  
* The service is *stateless* beyond `payments.db`; you can safely redeploy or run multiple front-end instances pointing to the same DB.
