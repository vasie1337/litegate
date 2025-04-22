import 'dotenv/config'
import Database from 'better-sqlite3'

const dbFile = process.env.DB_FILE || 'payments.db'
console.log(`[DB] opening ${dbFile}`)

const db = new Database(dbFile)
db.exec(`CREATE TABLE IF NOT EXISTS payments(
  id TEXT PRIMARY KEY,
  address TEXT UNIQUE,
  wif_enc TEXT,
  amount INTEGER,
  status TEXT,
  txid TEXT,
  created_at INTEGER,
  updated_at INTEGER
)`)
console.log('[DB] table ready')

export default db
