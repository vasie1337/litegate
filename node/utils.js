import 'dotenv/config'
import crypto from 'crypto'
import * as ecc from 'tiny-secp256k1'
import { address as btcAddress } from 'bitcoinjs-lib'
import { ECPairFactory } from 'ecpair'

const netParams = { messagePrefix:'\x19Litecoin Signed Message:\n', bech32:'ltc',
  bip32:{ public:0x019da462, private:0x019d9cfe }, pubKeyHash:0x30,
  scriptHash:0x32, wif:0xb0 }

const sha256 = b => crypto.createHash('sha256').update(b).digest()
const sh     = a => sha256(btcAddress.toOutputScript(a, netParams)).reverse().toString('hex')
const ECPair = ECPairFactory(ecc)

/* --- WIF encryption helpers -------------------------------------------- */
const keyHex = process.env.WIF_KEY || ''
if (keyHex.length !== 64) throw new Error('WIF_KEY must be 32‑byte hex')
const key = Buffer.from(keyHex, 'hex')

const encryptWIF = wif => {
  const iv     = crypto.randomBytes(12)
  const cipher = crypto.createCipheriv('aes-256-gcm', key, iv)
  const enc    = Buffer.concat([cipher.update(wif, 'utf8'), cipher.final()])
  const tag    = cipher.getAuthTag()
  return Buffer.concat([iv, tag, enc]).toString('hex')
}

const decryptWIF = hex => {
  const buf      = Buffer.from(hex, 'hex')
  const iv       = buf.slice(0, 12)
  const tag      = buf.slice(12, 28)
  const enc      = buf.slice(28)
  const decipher = crypto.createDecipheriv('aes-256-gcm', key, iv)
  decipher.setAuthTag(tag)
  return Buffer.concat([decipher.update(enc), decipher.final()]).toString('utf8')
}

export { sha256, sh, ECPair, netParams, encryptWIF, decryptWIF }
