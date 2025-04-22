use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

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
}

impl Db {
    pub fn open(path: &str) -> Self {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS payments(
                id TEXT PRIMARY KEY,
                address TEXT UNIQUE,
                wif_enc TEXT,
                amount REAL,
                status TEXT,
                created_at INTEGER,
                updated_at INTEGER
            )",
        )
        .unwrap();
        Self(Arc::new(Mutex::new(conn)))
    }
    pub fn insert(&self, p: &Payment) {
        let c = self.0.lock().unwrap();
        c.execute(
            "INSERT INTO payments(id,address,wif_enc,amount,status,created_at,updated_at)
             VALUES(?,?,?,?,?,strftime('%s','now'),strftime('%s','now'))",
            params![p.id, p.address, p.wif_enc, p.amount, "pending"],
        )
        .unwrap();
    }
    pub fn find(&self, id: &str) -> Option<Payment> {
        let c = self.0.lock().unwrap();
        c.query_row(
            "SELECT id,address,wif_enc,amount,status,created_at,updated_at
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
                })
            },
        )
        .ok()
    }
    pub fn all(&self) -> Vec<Payment> {
        let c = self.0.lock().unwrap();
        let mut stmt = c
            .prepare("SELECT id,address,wif_enc,amount,status,created_at,updated_at FROM payments")
            .unwrap();
        stmt.query_map([], |r| {
            Ok(Payment {
                id: r.get(0)?,
                address: r.get(1)?,
                wif_enc: r.get(2)?,
                amount: r.get(3)?,
                status: r.get(4)?,
                created_at: r.get(5)?,
                updated_at: r.get(6)?,
            })
        })
        .unwrap()
        .map(|x| x.unwrap())
        .collect()
    }
    pub fn mark_completed(&self, id: &str) {
        let c = self.0.lock().unwrap();
        c.execute(
            "UPDATE payments SET status='completed',updated_at=strftime('%s','now') WHERE id=?",
            [id],
        )
        .unwrap();
    }
}
