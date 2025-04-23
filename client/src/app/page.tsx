'use client'
import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { Clock, ArrowRight, Info, Check, CreditCard, AlertTriangle } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'

const API = process.env.NEXT_PUBLIC_API_URL
const EXPIRATION_OPTIONS = [
  { label: '15m', value: '900' },
  { label: '1h', value: '3600' },
  { label: '24h', value: '86400' },
  { label: "Never", value: "0" }
]
const INFO_CARDS = [
  {
    title: "Network Fee",
    value: "~0.001 LTC",
    description: "Average transaction fee",
    icon: "litecoin",
  },
  {
    title: "Confirmations",
    value: "2 required",
    description: "For payment verification",
    icon: "check",
  },
  {
    title: "Processing",
    value: "~2.5 min",
    description: "Average confirmation time",
    icon: "clock",
  }
]

// Components
const LitecoinIcon = ({ className = "h-6 w-6" }) => (
  <svg
    viewBox="0 0 82.6 82.6"
    xmlns="http://www.w3.org/2000/svg"
    className={className}
  >
    <circle cx="41.3" cy="41.3" r="36.83" style={{ fill: "#fff" }} />
    <path
      d="M41.3,0A41.3,41.3,0,1,0,82.6,41.3h0A41.18,41.18,0,0,0,41.54,0ZM42,42.7,37.7,57.2h23a1.16,1.16,0,0,1,1.2,1.12v.38l-2,6.9a1.49,1.49,0,0,1-1.5,1.1H23.2l5.9-20.1-6.6,2L24,44l6.6-2,8.3-28.2a1.51,1.51,0,0,1,1.5-1.1h8.9a1.16,1.16,0,0,1,1.2,1.12v.38L43.5,38l6.6-2-1.4,4.8Z"
      style={{ fill: "#345d9d" }}
    />
  </svg>
)

const InfoTooltip = ({ text }: any) => (
  <div className="relative group ml-2">
    <Info className="h-4 w-4 text-neutral-500" />
    <motion.div
      className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 hidden group-hover:block bg-neutral-700 text-neutral-300 text-xs p-2 rounded shadow-lg w-48 z-10"
      initial={{ opacity: 0, y: 5 }}
      whileInView={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2 }}
    >
      {text}
    </motion.div>
  </div>
)

const BackgroundDecoration = () => (
  <div className="absolute top-0 left-0 w-full h-full overflow-hidden pointer-events-none">
    <motion.div
      className="absolute top-20 left-5 w-32 h-32 rounded-full bg-[#345d9d]/10 blur-3xl"
      animate={{
        scale: [1, 1.2, 1],
        x: [0, 20, 0],
        y: [0, 30, 0],
      }}
      transition={{
        duration: 10,
        repeat: Infinity,
        repeatType: "reverse"
      }}
    />
    <motion.div
      className="absolute bottom-20 right-10 w-64 h-64 rounded-full bg-[#345d9d]/10 blur-3xl"
      animate={{
        scale: [1, 1.4, 1],
        x: [0, -30, 0],
        y: [0, -20, 0],
      }}
      transition={{
        duration: 15,
        repeat: Infinity,
        repeatType: "reverse",
        delay: 1
      }}
    />
  </div>
)

const InfoCard = ({ title, value, description, icon }: any) => (
  <motion.div
    className="bg-neutral-800/40 backdrop-blur-sm rounded-xl border border-neutral-700/50 p-4 shadow-lg"
    initial={{ y: 20, opacity: 0 }}
    animate={{ y: 0, opacity: 1 }}
    whileHover={{
      y: -5,
      boxShadow: "0 10px 25px -5px rgba(0, 0, 0, 0.3)",
      borderColor: "rgba(100, 150, 200, 0.3)"
    }}
  >
    <div className="bg-neutral-800/80 rounded-lg p-3 w-12 h-12 flex items-center justify-center mb-3">
      {icon === 'litecoin' ? <LitecoinIcon className="h-6 w-6" /> :
        icon === 'check' ? <Check className="h-6 w-6 text-green-400" /> :
          <Clock className="h-6 w-6 text-neutral-400" />}
    </div>
    <p className="text-xs text-neutral-400 mb-1">{title}</p>
    <p className="text-sm font-medium text-neutral-100 mb-1">{value}</p>
    <p className="text-xs text-neutral-500">{description}</p>
  </motion.div>
)

const SubmitButton = ({ formState, handleSubmit }: any) => (
  <AnimatePresence mode="wait">
    {formState === 'idle' && (
      <motion.button
        key="submit"
        type="submit"
        onClick={handleSubmit}
        className="w-full px-6 py-4 bg-gradient-to-r from-[#345d9d] to-[#25426e] hover:from-[#3a68b0] hover:to-[#2a4a7d] text-white rounded-xl text-base font-medium transition-all duration-200 flex items-center justify-center group shadow-lg shadow-[#345d9d]/20"
        whileHover={{ scale: 1.02 }}
        whileTap={{ scale: 0.98 }}
        exit={{ opacity: 0, y: 20 }}
      >
        <span>Generate Payment Link</span>
        <motion.div
          className="ml-2"
          initial={{ x: 0 }}
          animate={{ x: [0, 5, 0] }}
          transition={{ duration: 1.5, repeat: Infinity, repeatType: "loop", ease: "easeInOut" }}
        >
          <ArrowRight className="h-5 w-5" />
        </motion.div>
      </motion.button>
    )}

    {formState === 'loading' && (
      <motion.div
        key="loading"
        className="w-full px-6 py-4 bg-gradient-to-r from-[#345d9d] to-[#25426e] text-white rounded-xl text-base font-medium flex items-center justify-center"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: 20 }}
      >
        <motion.div
          className="h-5 w-5 border-2 border-white border-t-transparent rounded-full"
          animate={{ rotate: 360 }}
          transition={{ duration: 1, repeat: Infinity, ease: "linear" }}
        />
        <span className="ml-3">Processing...</span>
      </motion.div>
    )}

    {formState === 'success' && (
      <motion.div
        key="success"
        className="w-full px-6 py-4 bg-green-700 text-white rounded-xl text-base font-medium flex items-center justify-center"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: 20 }}
      >
        <Check className="h-5 w-5 mr-2" />
        <span>Payment Link Created!</span>
      </motion.div>
    )}

    {formState === 'error' && (
      <motion.div
        key="error"
        className="w-full px-6 py-4 bg-red-700 text-white rounded-xl text-base font-medium flex items-center justify-center"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0, x: [0, -5, 5, -5, 5, 0] }}
        transition={{ x: { delay: 0.3, duration: 0.5 } }}
        exit={{ opacity: 0, y: 20 }}
      >
        <AlertTriangle className="h-5 w-5 mr-2" />
        <span>Failed to create payment</span>
      </motion.div>
    )}
  </AnimatePresence>
)

export default function Page() {
  const [amount, setAmount] = useState('')
  const [ttl, setTtl] = useState('3600')
  const [formState, setFormState] = useState('idle')
  const router = useRouter()

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault()
    setFormState('loading')

    try {
      const r = await fetch(`${API}/payments`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ amount: Number(amount), ttl: Number(ttl) })
      })

      if (!r.ok) {
        throw new Error('Payment creation failed')
      }

      const { id } = await r.json()
      setFormState('success')

      setTimeout(() => {
        router.push(`/${id}`)
      }, 1000)
    } catch (error) {
      setFormState('error')
      setTimeout(() => setFormState('idle'), 3000)
    }
  }

  return (
    <>
      <motion.div
        className="min-h-screen bg-gradient-to-br from-neutral-900 via-neutral-900 to-[#1c2b52] p-6 overflow-hidden relative"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.5 }}
      >
        <BackgroundDecoration />

        <div className="max-w-lg mx-auto relative z-10">
          <motion.div
            className="flex items-center justify-center mb-10"
            initial={{ y: -20 }}
            animate={{ y: 0 }}
            transition={{ delay: 0.1, type: "spring", stiffness: 200 }}
          >
            <LitecoinIcon className="h-10 w-10 mr-6" />
            <div>
              <h1 className="text-3xl font-bold text-neutral-100 tracking-tight">LiteGate</h1>
              <p className="text-neutral-400 text-sm">Fast, secure cryptocurrency transactions</p>
            </div>
          </motion.div>

          <motion.div
            className="bg-neutral-800/80 backdrop-blur-sm rounded-xl shadow-2xl border border-neutral-700/70 overflow-hidden mb-8"
            initial={{ y: 20, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            transition={{ delay: 0.2, duration: 0.4 }}
          >
            <div className="bg-gradient-to-r from-neutral-800 to-neutral-700 px-6 py-4 border-b border-neutral-600 flex justify-between items-center">
              <h2 className="text-lg font-semibold text-neutral-100 flex items-center">
                Create New Payment
              </h2>
              <motion.div
                className="px-3 py-1 bg-neutral-900/80 rounded-full text-xs font-medium text-neutral-300 flex items-center"
                whileHover={{ scale: 1.05 }}
              >
                <Clock className="h-3 w-3 mr-1" />
                Auto-expires
              </motion.div>
            </div>

            <div className="px-6 py-8">
              <form onSubmit={handleSubmit} className="space-y-7">
                {/* Amount Input */}
                <motion.div
                  className="space-y-2"
                  initial={{ x: -10, opacity: 0 }}
                  animate={{ x: 0, opacity: 1 }}
                  transition={{ delay: 0.3 }}
                >
                  <label className="text-sm font-medium text-neutral-300 flex items-center">
                    LTC Amount
                    <InfoTooltip text="Enter the amount in Litecoin that you want to receive" />
                  </label>
                  <div className="relative">
                    <div className="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none z-10">
                      <LitecoinIcon className="h-5 w-5" />
                    </div>
                    <motion.input
                      type="number"
                      step="any"
                      placeholder="0.00"
                      value={amount}
                      onChange={e => setAmount(e.target.value)}
                      required
                      className="w-full pl-12 pr-4 py-4 bg-neutral-700/50 backdrop-blur-sm border border-neutral-600 rounded-xl text-neutral-100 placeholder-neutral-400 focus:outline-none focus:ring-2 focus:ring-[#345d9d]/50 transition-all duration-200 shadow-inner"
                      whileFocus={{ scale: 1.01 }}
                      transition={{ type: "spring", stiffness: 300 }}
                    />
                    <motion.div
                      className="absolute right-3 top-1/2 -translate-y-1/2 text-xs font-medium text-neutral-400 px-2 py-1 bg-neutral-800/70 rounded-md"
                      animate={{ opacity: amount ? 1 : 0.5 }}
                    >
                      LTC
                    </motion.div>
                  </div>
                </motion.div>

                {/* Expiration Input */}
                <motion.div
                  className="space-y-2"
                  initial={{ x: -10, opacity: 0 }}
                  animate={{ x: 0, opacity: 1 }}
                  transition={{ delay: 0.4 }}
                >
                  <label className="text-sm font-medium text-neutral-300 flex items-center">
                    Payment Expiration
                    <InfoTooltip text="Specify how many seconds the payment will remain valid" />
                  </label>
                  <div className="relative">
                    <div className="absolute inset-y-0 left-0 pl-4 flex items-center z-10 pointer-events-none">
                      <Clock className="h-5 w-5 text-neutral-500" />
                    </div>
                    <motion.input
                      type="number"
                      placeholder="3600"
                      value={ttl}
                      onChange={e => setTtl(e.target.value)}
                      required
                      className="w-full pl-12 pr-4 py-4 bg-neutral-700/50 backdrop-blur-sm border border-neutral-600 rounded-xl text-neutral-100 placeholder-neutral-400 focus:outline-none focus:ring-2 focus:ring-[#345d9d]/50 transition-all duration-200 shadow-inner"
                      whileFocus={{ scale: 1.01 }}
                      transition={{ type: "spring", stiffness: 300 }}
                    />
                    <motion.div
                      className="absolute right-3 top-1/2 -translate-y-1/2 text-xs font-medium text-neutral-400 px-2 py-1 bg-neutral-800/70 rounded-md"
                    >
                      seconds
                    </motion.div>
                  </div>
                </motion.div>

                <div className="flex items-center -mt-4 justify-between">
                  <div className="flex gap-2">
                    {EXPIRATION_OPTIONS.map(option => (
                      <motion.button
                        key={option.value}
                        type="button"
                        className={`text-xs px-2 py-1 rounded-md transition-colors ${ttl === option.value
                          ? 'bg-[#345d9d]/30 text-[#4b7cbd] border border-[#345d9d]/50'
                          : 'bg-neutral-800/70 text-neutral-400 border border-neutral-700 hover:border-neutral-600'
                          }`}
                        onClick={() => setTtl(option.value)}
                        whileHover={{ scale: 1.05 }}
                        whileTap={{ scale: 0.95 }}
                      >
                        {option.label}
                      </motion.button>
                    ))}
                  </div>
                </div>
              </form>
            </div>
          </motion.div>

          <motion.div
            initial={{ y: 10, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            transition={{ delay: 0.5 }}
          >
            <SubmitButton formState={formState} handleSubmit={handleSubmit} />
          </motion.div>
        </div>
      </motion.div>

      <motion.div
        className="bg-neutral-800/80 border-t border-neutral-700/80 px-6 py-4"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.6 }}
      >
        <div className="flex items-center justify-between">
          <div className="flex items-center text-xs text-neutral-400">
            <CreditCard className="h-4 w-4 mr-2 text-[#345d9d]" />
            Processed via the Litecoin network
          </div>
          <div className="flex items-center gap-2">
            <div className="h-2 w-2 rounded-full bg-green-500 animate-pulse"></div>
            <span className="text-xs text-green-500">Network Online</span>
          </div>
        </div>
      </motion.div>

      <div className="grid grid-cols-3 gap-6">
        {INFO_CARDS.map((item, index) => (
          <InfoCard
            key={index}
            title={item.title}
            value={item.value}
            description={item.description}
            icon={item.icon}
          />
        ))}
      </div>
    </>
  )
}