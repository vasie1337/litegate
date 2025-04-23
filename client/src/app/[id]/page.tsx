'use client'
import { useState, useEffect } from 'react'
import { useParams } from 'next/navigation'
import { motion, AnimatePresence } from 'framer-motion'
import { Clock, Copy, Check, QrCode, AlertTriangle } from 'lucide-react'
import { QRCodeSVG } from 'qrcode.react'

const API = process.env.NEXT_PUBLIC_API_URL

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



export default function Page() {
    const { id } = useParams()
    const [data, setData] = useState(null)
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState(false)
    const [copied, setCopied] = useState(false)
    const [lastRefresh, setLastRefresh] = useState(0)
    const [refreshPulse, setRefreshPulse] = useState(false)

    const formatDate = (timestamp) => {
        if (!timestamp) return '';
        const date = new Date(timestamp);
        return new Intl.DateTimeFormat('en-US', {
            month: 'short',
            day: 'numeric',
            year: 'numeric',
            hour: 'numeric',
            minute: 'numeric',
            second: 'numeric',
            hour12: true
        }).format(date);
    };

    const getTimeData = (created_at, expires_at) => {
        if (!created_at || !expires_at) {
            console.log('Missing date values:', { created_at, expires_at });
            return {
                remainingMs: 0,
                totalDurationMs: 0,
                expired: true
            };
        }

        // Parse timestamps carefully
        const now = new Date().getTime();

        // Handle different timestamp formats
        let createdTime, expiryTime;

        // Handle numeric timestamps
        if (typeof created_at === 'number' && typeof expires_at === 'number') {
            // Check if seconds instead of milliseconds (Unix timestamps)
            if (created_at < 10000000000) {
                createdTime = created_at * 1000;
            } else {
                createdTime = created_at;
            }

            if (expires_at < 10000000000) {
                expiryTime = expires_at * 1000;
            } else {
                expiryTime = expires_at;
            }
        } else {
            // String handling
            createdTime = new Date(created_at).getTime();
            expiryTime = new Date(expires_at).getTime();

            // Debug invalid dates
            if (isNaN(createdTime) || isNaN(expiryTime)) {
                console.log('Invalid date parsing:', {
                    created_at,
                    expires_at,
                    createdTime: isNaN(createdTime) ? 'Invalid' : createdTime,
                    expiryTime: isNaN(expiryTime) ? 'Invalid' : expiryTime
                });

                // Fallback for demo/testing - set expiry 15 minutes from now
                if (isNaN(createdTime)) createdTime = now - 300000; // 5 minutes ago
                if (isNaN(expiryTime)) expiryTime = now + 900000;   // 15 minutes ahead
            }
        }

        const totalDuration = expiryTime - createdTime;
        const remaining = expiryTime - now;

        console.log('Time calculation:', {
            now,
            createdTime,
            expiryTime,
            totalDuration,
            remaining,
            remainingFormatted: `${Math.floor(Math.max(0, remaining) / 60000)}:${Math.floor((Math.max(0, remaining) % 60000) / 1000).toString().padStart(2, '0')}`
        });

        return {
            remainingMs: Math.max(0, remaining),
            totalDurationMs: Math.max(1, totalDuration), // Prevent division by zero
            expired: remaining <= 0
        };
    };

    const getStatusDetails = (status) => {
        if (!status) return {
            color: '#737373',
            bgColor: '#262626',
            label: 'Unknown',
            icon: <AlertTriangle className="h-4 w-4" />
        };

        switch (status.toLowerCase()) {
            case 'pending':
                return {
                    color: '#eab308',
                    bgColor: 'rgba(113, 63, 18, 0.3)',
                    label: 'Pending Payment',
                    icon: <Clock className="h-4 w-4" />
                };
            case 'completed':
                return {
                    color: '#22c55e',
                    bgColor: 'rgba(20, 83, 45, 0.3)',
                    label: 'Payment Confirmed',
                    icon: <Check className="h-4 w-4" />
                };
            case 'expired':
                return {
                    color: '#ef4444',
                    bgColor: 'rgba(127, 29, 29, 0.3)',
                    label: 'Expired',
                    icon: <AlertTriangle className="h-4 w-4" />
                };
            default:
                return {
                    color: '#737373',
                    bgColor: '#262626',
                    label: status,
                    icon: <AlertTriangle className="h-4 w-4" />
                };
        }
    };

    const copyToClipboard = (text) => {
        navigator.clipboard.writeText(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    useEffect(() => {
        if (!data) return;

        const timer = setInterval(() => {
            setLastRefresh(prev => prev + 1);
        }, 1000);

        return () => clearInterval(timer);
    }, [data]);

    useEffect(() => {
        if (!id) return;

        const load = async () => {
            try {
                // For debugging - check if we're using a test environment
                if (!API || API === 'undefined') {
                    console.warn('API URL is not defined. Using mock data for testing.');

                    // Mock data for testing the UI when API is unavailable
                    const now = new Date();
                    const mockData = {
                        id: id,
                        amount: '0.25',
                        usdAmount: '19.99',
                        address: 'MKL45asdJK23lkj5235lkjfsSADFja',
                        status: 'pending',
                        created_at: new Date(now.getTime() - 300000).toISOString(), // 5 minutes ago
                        expires_at: new Date(now.getTime() + 900000).toISOString()  // 15 minutes from now
                    };

                    console.log('Using mock data:', mockData);
                    setData(mockData);
                    setError(false);
                } else {
                    const r = await fetch(`${API}/payments/${id}`);
                    if (r.ok) {
                        const jsonData = await r.json();
                        console.log('API response:', jsonData);
                        setData(jsonData);
                        setError(false);
                    } else {
                        console.error('API error status:', r.status);
                        setError(true);
                    }
                }

                setRefreshPulse(true);
                setTimeout(() => setRefreshPulse(false), 1000);
                setLastRefresh(0);
            } catch (e) {
                console.error('Error loading data:', e);
                setError(true);
            } finally {
                setLoading(false);
            }
        }

        load();

        const t = setInterval(() => {
            load();
        }, 5000);

        return () => clearInterval(t);
    }, [id]);

    const timeData = data ? getTimeData(data.created_at, data.expires_at) : {
        remainingMs: 0,
        totalDurationMs: 0,
        expired: true
    };

    const statusDetails = data ? getStatusDetails(data.status) : getStatusDetails(null);

    return (
        <motion.div
            className="min-h-screen bg-gradient-to-br from-neutral-900 via-neutral-900 to-[#1c2b52] p-6 overflow-hidden relative"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.5 }}
        >
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

            <div className="max-w-lg mx-auto relative z-10">
                <motion.div
                    className="flex items-center justify-center mb-10"
                    initial={{ y: -20 }}
                    animate={{ y: 0 }}
                    transition={{ delay: 0.1, type: "spring", stiffness: 200 }}
                >
                    <LitecoinIcon className="h-10 w-10 mr-4" />
                    <div>
                        <h1 className="text-3xl font-bold text-neutral-100 tracking-tight">LiteGate</h1>
                        <div className="flex items-center">
                            <p className="text-neutral-400 text-sm">Payment ID: {id}</p>
                        </div>
                    </div>
                </motion.div>

                <AnimatePresence>
                    {loading && (
                        <motion.div
                            className="bg-neutral-800/80 backdrop-blur-sm rounded-xl shadow-2xl border border-neutral-700/70 p-8 flex flex-col items-center justify-center mb-6"
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            exit={{ opacity: 0 }}
                        >
                            <motion.div
                                className="h-12 w-12 border-4 border-[#345d9d] border-t-transparent rounded-full mb-4"
                                animate={{ rotate: 360 }}
                                transition={{ duration: 1.5, repeat: Infinity, ease: "linear" }}
                            />
                            <p className="text-neutral-300 font-medium">Loading payment details...</p>
                        </motion.div>
                    )}
                </AnimatePresence>

                <AnimatePresence>
                    {!loading && error && (
                        <motion.div
                            className="bg-red-900/20 backdrop-blur-sm rounded-xl shadow-2xl border border-red-700/30 p-6 mb-6"
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            exit={{ opacity: 0, y: -20 }}
                        >
                            <div className="flex items-start">
                                <div className="bg-red-900/30 p-3 rounded-lg mr-4">
                                    <AlertTriangle className="h-6 w-6 text-red-500" />
                                </div>
                                <div>
                                    <h2 className="text-lg font-semibold text-red-500 mb-2">Error Loading Payment</h2>
                                    <p className="text-neutral-300 mb-4">We couldn't load the payment details. Please check the payment ID and try again.</p>
                                </div>
                            </div>
                        </motion.div>
                    )}
                </AnimatePresence>

                <AnimatePresence>
                    {!loading && !error && data && (
                        <motion.div
                            className="space-y-6"
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            exit={{ opacity: 0, y: -20 }}
                            transition={{ duration: 0.4 }}
                        >
                            <motion.div
                                className="bg-neutral-800/80 backdrop-blur-sm rounded-xl shadow-2xl border border-neutral-700/70 overflow-hidden"
                                initial={{ y: 20, opacity: 0 }}
                                animate={{ y: 0, opacity: 1 }}
                                transition={{ delay: 0.2 }}
                            >
                                <div style={{ backgroundColor: statusDetails.bgColor }} className="px-6 py-4 border-b border-neutral-700/80">
                                    <div className="flex justify-between items-center">
                                        <h2 className="text-lg font-semibold text-neutral-100 flex items-center">
                                            Payment Status
                                        </h2>
                                        <motion.div
                                            style={{
                                                backgroundColor: statusDetails.bgColor,
                                                color: statusDetails.color
                                            }}
                                            className="px-3 py-1 rounded-full text-xs font-medium flex items-center"
                                        >
                                            {statusDetails.icon}
                                            <span className="ml-1">{statusDetails.label}</span>
                                        </motion.div>
                                    </div>
                                </div>

                                <div className="p-6">
                                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                                        <div className="space-y-4">
                                            <div>
                                                <p className="text-sm font-medium text-neutral-400 mb-1">Amount</p>
                                                <div className="flex items-center">
                                                    <LitecoinIcon className="h-5 w-5 mr-2" />
                                                    <p className="text-xl font-semibold text-neutral-100">{data.amount} LTC</p>
                                                </div>
                                                <div className="flex flex-col mt-1">
                                                    <p className="text-sm text-neutral-500">
                                                        <span className="text-neutral-400">Received:</span> {data.received} LTC
                                                    </p>
                                                </div>
                                            </div>

                                            <div>
                                                <p className="text-sm font-medium text-neutral-400 mb-1">Created</p>
                                                <p className="text-sm text-neutral-300">
                                                    {data.created_at && new Date(
                                                        typeof data.created_at === 'number' && data.created_at < 10000000000
                                                            ? data.created_at * 1000  // Convert seconds to milliseconds
                                                            : data.created_at
                                                    ).toLocaleString('en-US', {
                                                        month: 'short',
                                                        day: 'numeric',
                                                        year: 'numeric',
                                                        hour: 'numeric',
                                                        minute: 'numeric',
                                                        second: 'numeric',
                                                        hour12: true
                                                    })}
                                                </p>
                                            </div>

                                            <div className="bg-neutral-900 backdrop-blur-sm rounded-lg p-4 border border-neutral-700/50">
                                                <div className="flex items-center">
                                                    <div className="relative w-8 h-8 mr-3">
                                                        <svg viewBox="0 0 24 24" className="w-full h-full">
                                                            <circle
                                                                cx="12"
                                                                cy="12"
                                                                r="10"
                                                                fill="none"
                                                                stroke="#3f3f46"
                                                                strokeWidth="2"
                                                            />
                                                            <circle
                                                                cx="12"
                                                                cy="12"
                                                                r="10"
                                                                fill="none"
                                                                stroke="#345d9d"
                                                                strokeWidth="2"
                                                                strokeDasharray={2 * Math.PI * 10}
                                                                strokeDashoffset={(2 * Math.PI * 10) * (1 - (timeData.remainingMs / timeData.totalDurationMs))}
                                                                transform="rotate(-90 12 12)"
                                                            />
                                                        </svg>
                                                    </div>
                                                    <div>
                                                        <p className="text-sm font-medium text-neutral-300">Expiration time</p>
                                                        <p className="text-sm text-neutral-400">
                                                            {`${Math.floor(timeData.remainingMs / 60000)}:${Math.floor((timeData.remainingMs % 60000) / 1000).toString().padStart(2, '0')}`}
                                                        </p>
                                                    </div>
                                                </div>
                                            </div>

                                            <motion.div
                                                className="bg-neutral-900 rounded-lg p-3 border border-neutral-700/50 backdrop-blur-sm"
                                                initial={{ opacity: 0, y: 5 }}
                                                animate={{ opacity: 1, y: 0 }}
                                                transition={{ delay: 0.5 }}
                                            >
                                                <div className="flex items-center justify-between mb-2">
                                                    <p className="text-xs text-neutral-500 font-medium">Confirmations</p>
                                                    <span className="text-base font-bold text-neutral-100">
                                                        {data.confirmations || 0}<span className="text-xs text-neutral-500 ml-1">/2</span>
                                                    </span>
                                                </div>

                                                <div className="w-full bg-neutral-800/50 h-2 rounded-full overflow-hidden mb-2">
                                                    <motion.div
                                                        className="h-full bg-[#345d9d]"
                                                        style={{ width: `${Math.min(100, ((data.confirmations || 0) / 2) * 100)}%` }}
                                                        initial={{ width: 0 }}
                                                        animate={{ width: `${Math.min(100, ((data.confirmations || 0) / 2) * 100)}%` }}
                                                        transition={{ delay: 0.6, duration: 0.8 }}
                                                    />
                                                </div>

                                                <div className="flex justify-between items-center">
                                                    {[...Array(2)].map((_, i) => (
                                                        <motion.div
                                                            key={i}
                                                            className="flex flex-col items-center"
                                                            initial={{ opacity: 0 }}
                                                            animate={{ opacity: 1 }}
                                                            transition={{ delay: 0.7 + (i * 0.1) }}
                                                        >
                                                            <div className={`w-2 h-2 rounded-full ${i < (data.confirmations || 0) ? 'bg-[#345d9d]' : 'bg-neutral-700'}`} />
                                                            <span className="text-[10px] mt-1 text-neutral-500">
                                                                {i === 0 ? 'Pending' : 'Confirmed'}
                                                            </span>
                                                        </motion.div>
                                                    ))}
                                                </div>

                                                {(data.confirmations || 0) === 0 && (
                                                    <p className="text-[10px] text-neutral-400 mt-2 text-center">
                                                        Waiting for network confirmation...
                                                    </p>
                                                )}
                                            </motion.div>
                                        </div>

                                        <div className='space-y-8'>
                                            <p className="text-sm font-medium text-neutral-400 mb-4">Payment Address</p>

                                            <div className="flex flex-col items-center justify-center">
                                                {data.address ? (
                                                    <motion.div
                                                        className="bg-white p-4 rounded-lg shadow-lg"
                                                        initial={{ scale: 0.9, opacity: 0 }}
                                                        animate={{ scale: 1, opacity: 1 }}
                                                        transition={{ delay: 0.4 }}
                                                    >
                                                        <div className="text-center">
                                                            <QRCodeSVG
                                                                value={`litecoin:${data.address}?amount=${data.amount}`}
                                                                size={128}
                                                                level="M"
                                                                bgColor="#ffffff"
                                                                fgColor="#345d9d"
                                                                includeMargin={true}
                                                                className="mx-auto"
                                                            />
                                                            <p className="text-neutral-800 text-xs mt-2">Scan to pay</p>
                                                        </div>
                                                    </motion.div>
                                                ) : (
                                                    <div className="flex items-center justify-center h-full">
                                                        <p className="text-neutral-500 text-center">QR code not available</p>
                                                    </div>
                                                )}
                                            </div>

                                            <div className="bg-neutral-700/50 backdrop-blur-sm rounded-lg p-3 flex items-center justify-between group">
                                                <p className="text-neutral-300 text-sm font-mono truncate">
                                                    {data.address || "Not available"}
                                                </p>
                                                {data.address && (
                                                    <motion.button
                                                        onClick={() => copyToClipboard(data.address)}
                                                        className="text-neutral-400 hover:text-neutral-200 p-1"
                                                        whileHover={{ scale: 1.1 }}
                                                        whileTap={{ scale: 0.9 }}
                                                    >
                                                        {copied ? <Check className="h-4 w-4 text-green-500" /> : <Copy className="h-4 w-4" />}
                                                    </motion.button>
                                                )}
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                <div className="bg-neutral-800/80 border-t border-neutral-700/80 px-4 py-3">
                                    <div className="flex items-center text-xs text-neutral-400 justify-end">
                                        {!loading && !error && (
                                            <motion.div
                                                className="ml-3 flex items-center text-xs text-neutral-300"
                                                animate={{ opacity: refreshPulse ? 1 : 0.7 }}
                                            >
                                                <motion.div
                                                    className="h-2 w-2 rounded-full bg-green-500 mr-2"
                                                    animate={{ scale: refreshPulse ? [1, 1.5, 1] : 1 }}
                                                    transition={{ duration: 0.5 }}
                                                />
                                                <span>refreshed {lastRefresh}s ago</span>
                                            </motion.div>
                                        )}
                                    </div>
                                </div>
                            </motion.div>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>
        </motion.div>
    )
}