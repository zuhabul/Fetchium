'use client'

import { useState, useRef, useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { Shield, Eye, EyeOff, Loader2, AlertCircle } from 'lucide-react'

type Step = 'credentials' | 'totp'

export default function LoginPage() {
  const router = useRouter()
  const [step, setStep] = useState<Step>('credentials')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [totpCode, setTotpCode] = useState('')
  const [showPass, setShowPass] = useState(false)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const totpRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (step === 'totp') totpRef.current?.focus()
  }, [step])

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setError('')
    setLoading(true)

    try {
      const body: Record<string, string> = { email, password }
      if (step === 'totp') body.totp_code = totpCode

      const res = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      })

      const data = await res.json()

      if (res.status === 202 && data.totp_required) {
        setStep('totp')
        return
      }

      if (!res.ok) {
        setError(step === 'totp' ? 'Invalid code — try again' : 'Invalid email or password')
        return
      }

      router.push('/overview')
      router.refresh()
    } catch {
      setError('Connection error — try again')
    } finally {
      setLoading(false)
    }
  }

  // Auto-submit TOTP when 6 digits entered
  useEffect(() => {
    if (step === 'totp' && totpCode.length === 6) {
      handleSubmit(new Event('submit') as unknown as React.FormEvent)
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [totpCode])

  return (
    <div className="min-h-screen bg-zinc-950 flex items-center justify-center p-4">
      <div className="w-full max-w-sm">
        {/* Logo */}
        <div className="flex flex-col items-center mb-8">
          <div className="flex items-center gap-2 mb-2">
            <div className="w-8 h-8 bg-zinc-800 rounded-lg flex items-center justify-center border border-zinc-700">
              <Shield className="w-4 h-4 text-zinc-300" />
            </div>
            <span className="text-lg font-semibold text-zinc-100 tracking-tight">Fetchium</span>
            <span className="text-xs font-medium bg-zinc-800 text-zinc-400 border border-zinc-700 px-1.5 py-0.5 rounded">
              ADMIN
            </span>
          </div>
          <p className="text-zinc-500 text-sm">
            {step === 'credentials' ? 'Internal operations console' : 'Two-factor authentication'}
          </p>
        </div>

        {/* Card */}
        <div className="bg-zinc-900 border border-zinc-800 rounded-xl p-6">
          <form onSubmit={handleSubmit} className="space-y-4">
            {step === 'credentials' ? (
              <>
                <div>
                  <label className="block text-xs font-medium text-zinc-400 mb-1.5">
                    Email address
                  </label>
                  <input
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    required
                    autoFocus
                    autoComplete="email"
                    placeholder="you@fetchium.com"
                    className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2.5 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500 transition-colors"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-zinc-400 mb-1.5">
                    Password
                  </label>
                  <div className="relative">
                    <input
                      type={showPass ? 'text' : 'password'}
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      required
                      autoComplete="current-password"
                      placeholder="••••••••"
                      className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2.5 pr-10 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500 transition-colors"
                    />
                    <button
                      type="button"
                      onClick={() => setShowPass(!showPass)}
                      className="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300 transition-colors"
                    >
                      {showPass ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
              </>
            ) : (
              <div>
                <label className="block text-xs font-medium text-zinc-400 mb-1.5">
                  Authenticator code
                </label>
                <input
                  ref={totpRef}
                  type="text"
                  inputMode="numeric"
                  pattern="[0-9]*"
                  maxLength={6}
                  value={totpCode}
                  onChange={(e) => setTotpCode(e.target.value.replace(/\D/g, ''))}
                  placeholder="000000"
                  className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-3 text-2xl text-center font-mono tracking-[0.5em] text-zinc-100 placeholder-zinc-700 focus:outline-none focus:border-zinc-500 focus:ring-1 focus:ring-zinc-500 transition-colors"
                />
                <p className="text-xs text-zinc-600 mt-2 text-center">
                  Open your authenticator app and enter the 6-digit code
                </p>
              </div>
            )}

            {/* Error */}
            {error && (
              <div className="flex items-center gap-2 bg-red-500/10 border border-red-500/20 rounded-lg px-3 py-2.5">
                <AlertCircle className="w-4 h-4 text-red-400 flex-shrink-0" />
                <p className="text-sm text-red-400">{error}</p>
              </div>
            )}

            <button
              type="submit"
              disabled={loading}
              className="w-full bg-zinc-100 hover:bg-white text-zinc-950 font-medium text-sm rounded-lg py-2.5 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
            >
              {loading ? (
                <><Loader2 className="w-4 h-4 animate-spin" /> Verifying…</>
              ) : step === 'credentials' ? (
                'Sign in'
              ) : (
                'Verify'
              )}
            </button>

            {step === 'totp' && (
              <button
                type="button"
                onClick={() => { setStep('credentials'); setTotpCode(''); setError('') }}
                className="w-full text-center text-xs text-zinc-600 hover:text-zinc-400 transition-colors py-1"
              >
                ← Back to password
              </button>
            )}
          </form>
        </div>

        <p className="text-center text-xs text-zinc-700 mt-4">
          Staff accounts only — no self-registration
        </p>
      </div>
    </div>
  )
}
