import 'dotenv/config'
import { Psbt, payments } from 'bitcoinjs-lib'
import db from './db.js'
import { rpc, feeSat } from './electrum.js'
import { netParams, ECPair, sh, decryptWIF } from './utils.js'

const MAIN_ADDRESS   = process.env.MAIN_ADDRESS
const CONFIRMATIONS  = +process.env.CONFIRMATIONS_REQUIRED || 2
const POLL_MS        = 10_000

const now   = () => db.prepare("SELECT strftime('%s','now') ts").get().ts
const confs = async h => !h ? 0 : (await rpc('blockchain.headers.subscribe')).height - h + 1

const processPayment = async p => {
  const bal = await rpc('blockchain.scripthash.get_balance', [sh(p.address)])
  if (bal.confirmed < p.amount) return

  const utxos = await rpc('blockchain.scripthash.listunspent', [sh(p.address)])
  const ready = await Promise.all(utxos.map(u => confs(u.height)))
  if (ready.every(c => c >= CONFIRMATIONS) && p.status !== 'completed')
    db.prepare("UPDATE payments SET status='completed',updated_at=? WHERE id=?")
      .run(now(), p.id)

  if (bal.confirmed === 0 && bal.unconfirmed === 0) return

  const spendable = utxos.filter((_, i) => ready[i] >= CONFIRMATIONS)
  if (!spendable.length) return

  const key  = ECPair.fromWIF(decryptWIF(p.wif_enc), netParams)
  const psbt = new Psbt({ network: netParams })
  let total  = 0
  for (const u of spendable) {
    total += u.value
    psbt.addInput({
      hash: u.tx_hash,
      index: u.tx_pos,
      witnessUtxo: {
        script: payments.p2wpkh({ pubkey:key.publicKey, network:netParams }).output,
        value: u.value
      }
    })
  }
  const fee  = await feeSat(psbt.__CACHE.__TX.virtualSize() + 68)
  const send = total - fee
  if (send <= 0) return

  psbt.addOutput({ address: MAIN_ADDRESS, value: send })
  psbt.signAllInputs(key).finalizeAllInputs()
  const txid = await rpc('blockchain.transaction.broadcast', [psbt.extractTransaction().toHex()])
  console.log(`[Sweeper] ${p.id} broadcast ${txid}`)
}

export default () => setInterval(
  () => db.prepare('SELECT * FROM payments').all()
        .forEach(p => processPayment(p).catch(console.error)),
  POLL_MS
)
