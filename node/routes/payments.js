import crypto from 'crypto'
import { Router } from 'express'
import { payments as btc } from 'bitcoinjs-lib'
import db from '../db.js'
import { ECPair, sh, netParams, encryptWIF } from '../utils.js'
import { rpc } from '../electrum.js'

const router = Router()

router.post('/payments', (req, res) => {
  const ltc = +req.body.amount
  if (!ltc) return res.status(400).end()

  const id      = crypto.randomUUID()
  const key     = ECPair.makeRandom({ network: netParams })
  const address = btc.p2wpkh({ pubkey: key.publicKey, network: netParams }).address
  const sats    = Math.round(ltc * 1e8)
  const wifEnc  = encryptWIF(key.toWIF())

  db.prepare(`INSERT INTO payments(id,address,wif_enc,amount,status,created_at,updated_at)
              VALUES(?,?,?,?,'pending',strftime('%s','now'),strftime('%s','now'))`)
    .run(id, address, wifEnc, sats)

  res.json({ id, address, amount: ltc })
})

router.get('/payments/:id', async (req, res) => {
  const p = db.prepare(`SELECT id,address,amount,status,created_at,updated_at
                        FROM payments WHERE id=?`)
              .get(req.params.id)
  if (!p) return res.status(404).end()

  const [bal, hdr, hist] = await Promise.all([
    rpc('blockchain.scripthash.get_balance', [sh(p.address)]),
    rpc('blockchain.headers.subscribe'),
    rpc('blockchain.scripthash.get_history', [sh(p.address)])
  ])

  const tip  = hdr.height
  const conf = hist.filter(h => h.height > 0)
                   .map(h => tip - h.height + 1)
                   .reduce((m, c) => Math.min(m, c), Infinity)
  const confirmations = Number.isFinite(conf) ? conf : 0
  const pos = n => (n < 0 ? 0 : n)
  const received = (pos(bal.confirmed) + pos(bal.unconfirmed)) / 1e8

  res.json({
    ...p,
    amount: p.amount / 1e8,
    confirmations,
    received
  })
})

export default router
