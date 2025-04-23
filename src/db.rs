use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, instrument};

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
    #[instrument(skip(path), err)]
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        debug!("Opening database connection at {}", path);
        let conn = Connection::open(path)?;

        debug!("Creating payments table if it doesn't exist");
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
        )?;

        info!("Database connection established successfully");
        Ok(Self(Arc::new(Mutex::new(conn))))
    }

    #[instrument(skip(self, p), fields(payment_id = %p.id))]
    pub fn insert(&self, p: &Payment) -> SqliteResult<()> {
        debug!("Inserting new payment record");
        let c = self.0.lock().unwrap();

        c.execute(
            "INSERT INTO payments(id,address,wif_enc,amount,status,created_at,updated_at)
             VALUES(?,?,?,?,?,strftime('%s','now'),strftime('%s','now'))",
            params![p.id, p.address, p.wif_enc, p.amount, "pending"],
        )?;

        info!("Payment record inserted successfully");
        Ok(())
    }

    #[instrument(skip(self), fields(payment_id = %id))]
    pub fn find(&self, id: &str) -> SqliteResult<Option<Payment>> {
        debug!("Looking up payment by ID");
        let c = self.0.lock().unwrap();

        let result = c.query_row(
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
        );

        match result {
            Ok(payment) => {
                info!("Payment found");
                Ok(Some(payment))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                debug!("No payment found with ID: {}", id);
                Ok(None)
            }
            Err(err) => {
                error!("Database error while finding payment: {}", err);
                Err(err)
            }
        }
    }

    #[instrument(skip(self))]
    pub fn all(&self) -> SqliteResult<Vec<Payment>> {
        debug!("Retrieving all payments");
        let c = self.0.lock().unwrap();

        let mut stmt = c.prepare(
            "SELECT id,address,wif_enc,amount,status,created_at,updated_at FROM payments",
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
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        info!("Retrieved {} payment records", payments.len());
        Ok(payments)
    }

    #[instrument(skip(self), fields(payment_id = %id))]
    pub fn mark_completed(&self, id: &str) -> SqliteResult<()> {
        debug!("Marking payment as completed");
        let c = self.0.lock().unwrap();

        let rows_affected = c.execute(
            "UPDATE payments SET status='completed',updated_at=strftime('%s','now') WHERE id=?",
            [id],
        )?;

        if rows_affected > 0 {
            info!("Payment marked as completed");
        } else {
            debug!("No payment found to mark as completed");
        }

        Ok(())
    }
}
