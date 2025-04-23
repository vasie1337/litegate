use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::instrument;

#[derive(Clone)]
pub struct Db(pub Arc<Mutex<Connection>>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payment {
    pub id: String,
    pub address: String,
    pub wif_enc: String,
    pub amount: f64,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub expires_at: i64,
}

impl Db {
    #[instrument(skip(path), err)]
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS payments(
                id TEXT PRIMARY KEY,
                address TEXT UNIQUE,
                wif_enc TEXT,
                amount REAL,
                status TEXT,
                created_at INTEGER,
                updated_at INTEGER,
                expires_at INTEGER
            )",
        )?;
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_payments_expires_at ON payments(expires_at)",
        )?;
        Ok(Self(Arc::new(Mutex::new(conn))))
    }

    #[instrument(skip(self, p), fields(payment_id = %p.id))]
    pub fn insert(&self, p: &Payment) -> SqliteResult<()> {
        let c = self.0.lock().unwrap();
        c.execute(
            "INSERT INTO payments(id,address,wif_enc,amount,status,created_at,updated_at,expires_at)
             VALUES(?,?,?,?,?,strftime('%s','now'),strftime('%s','now'),?)",
            params![p.id, p.address, p.wif_enc, p.amount, "pending", p.expires_at],
        )?;
        Ok(())
    }

    #[instrument(skip(self), fields(payment_id = %id))]
    pub fn find(&self, id: &str) -> SqliteResult<Option<Payment>> {
        let c = self.0.lock().unwrap();
        let result = c.query_row(
            "SELECT id,address,wif_enc,amount,status,created_at,updated_at,expires_at
             FROM payments WHERE id=?",
            [id],
            |r| {
                Ok(Payment {
                    id: r.get(0)?,
                    address: r.get(1)?,
                    wif_enc: r.get(2)?,
                    amount: r.get(3)?,
                    status: r.get(4)?,
                    created_at: r.get(5)?,
                    updated_at: r.get(6)?,
                    expires_at: r.get(7)?,
                })
            },
        );
        match result {
            Ok(p) => Ok(Some(p)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn all(&self) -> SqliteResult<Vec<Payment>> {
        let c = self.0.lock().unwrap();
        let mut stmt = c.prepare(
            "SELECT id,address,wif_enc,amount,status,created_at,updated_at,expires_at FROM payments",
        )?;
        let payments = stmt
            .query_map([], |r| {
                Ok(Payment {
                    id: r.get(0)?,
                    address: r.get(1)?,
                    wif_enc: r.get(2)?,
                    amount: r.get(3)?,
                    status: r.get(4)?,
                    created_at: r.get(5)?,
                    updated_at: r.get(6)?,
                    expires_at: r.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(payments)
    }

    pub fn mark_completed(&self, id: &str) -> SqliteResult<()> {
        let c = self.0.lock().unwrap();
        c.execute(
            "UPDATE payments SET status='completed',updated_at=strftime('%s','now') WHERE id=?",
            [id],
        )?;
        Ok(())
    }

    pub fn mark_expired(&self, id: &str) -> SqliteResult<()> {
        let c = self.0.lock().unwrap();
        c.execute(
            "UPDATE payments SET status='expired',updated_at=strftime('%s','now') WHERE id=? AND status='pending'",
            [id],
        )?;
        Ok(())
    }
}
