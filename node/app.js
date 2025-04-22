import 'dotenv/config'
import express from 'express'
import paymentsRouter from './routes/payments.js'
import startSweeper from './sweeper.js'

console.log('[Bootstrap] starting …')
console.log('[Bootstrap] node', process.version)
console.log('[Bootstrap] env', JSON.stringify({
    PORT: process.env.PORT,
    ELECTRUM_HOST: process.env.ELECTRUM_HOST,
    ELECTRUM_PORT: process.env.ELECTRUM_PORT,
    ELECTRUM_SSL: process.env.ELECTRUM_SSL,
    MAIN_ADDRESS: process.env.MAIN_ADDRESS,
    CONFIRMATIONS: process.env.CONFIRMATIONS,
    DB_FILE: process.env.DB_FILE
}, null, 2))

process.on('uncaughtException', e => console.error('[Fatal] uncaught', e))
process.on('unhandledRejection', e => console.error('[Fatal] unhandled', e))

const PORT = +process.env.PORT || 3000

const app = express()
app.use(express.json())
app.use('/', paymentsRouter)

startSweeper()

app.listen(PORT, () => console.log(`[Server] listening on ${PORT}`))
