'use client'
import { useState } from 'react'
import { useRouter } from 'next/navigation'

const API = process.env.NEXT_PUBLIC_API_URL

export default function Page() {
  const [amount, setAmount] = useState('')
  const [ttl, setTtl] = useState('')
  const router = useRouter()
  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const r = await fetch(`${API}/payments`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ amount: Number(amount), ttl: Number(ttl) })
    })
    if (!r.ok) return
    const { id } = await r.json()
    router.push(`/${id}`)
  }
  return (
    <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: 8, maxWidth: 240 }}>
      <input type="number" step="any" placeholder="LTC amount" value={amount} onChange={e => setAmount(e.target.value)} required />
      <input type="number" placeholder="TTL (seconds)" value={ttl} onChange={e => setTtl(e.target.value)} required />
      <button type="submit">Create Payment</button>
    </form>
  )
}
