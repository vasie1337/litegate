import 'dotenv/config'
import Electrum from 'electrum-client'

const { ELECTRUM_HOST, ELECTRUM_PORT, ELECTRUM_SSL } = process.env

let ecl
const connect = async () => {
  console.log(`[Electrum] connect ${ELECTRUM_HOST}:${ELECTRUM_PORT} ssl=${ELECTRUM_SSL}`)
  ecl = new Electrum(+ELECTRUM_PORT, ELECTRUM_HOST, ELECTRUM_SSL === 'true' ? 'tls' : 'tcp')
  await ecl.connect()
  await ecl.server_version('ltc-payments/1.0', '1.4')
  console.log('[Electrum] ready')
}
await connect()

const rpc = async (m, p = []) => {
  for (let i = 0; i < 3; i++) {
    try {
      console.log(`[RPC] ${m} ${JSON.stringify(p)}`)
      const r = await ecl.request(m, p)
      console.log(`[RPC] ${m} ok`)
      return r
    } catch (e) {
      console.error(`[RPC] ${m} fail ${i + 1} – ${e.message}`)
      await connect()
    }
  }
  throw new Error(`RPC ${m} failed`)
}

const feeSat = async v => {
  const est = await rpc('blockchain.estimatefee', [6]) || 0
  const fee = v * Math.max(Math.ceil(est * 1e8 / 1000), 1)
  console.log(`[Electrum] fee vsize=${v} sat=${fee}`)
  return fee
}

export { rpc, feeSat }
