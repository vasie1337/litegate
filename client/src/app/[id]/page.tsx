'use client'
import { useState, useEffect } from 'react'
import { useParams } from 'next/navigation'

const API = process.env.NEXT_PUBLIC_API_URL

export default function Page() {
    const { id } = useParams()
    const [data, setData] = useState(null)
    useEffect(() => {
        if (!id) return
        const load = async () => {
            const r = await fetch(`${API}/payments/${id}`)
            if (r.ok) setData(await r.json())
        }
        load()
        const t = setInterval(load, 5000)
        return () => clearInterval(t)
    }, [id])
    if (!data) return <div>Loading…</div>
    return <pre>{JSON.stringify(data, null, 2)}</pre>
}
