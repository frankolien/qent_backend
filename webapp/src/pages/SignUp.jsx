import { useState, useRef } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';
import { ArrowRight, ArrowLeft, Shield, Car, Star } from 'lucide-react';
import { useAuth } from '../hooks/useAuth';
import { signUp } from '../utils/api';
import api from '../utils/api';
import Logo from '../components/Logo';

export default function SignUp() {
  const [form, setForm] = useState({ full_name: '', email: '', password: '', country: 'Nigeria' });
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [step, setStep] = useState('details');
  const [code, setCode] = useState(['', '', '', '']);
  const [resending, setResending] = useState(false);
  const [maskedEmail, setMaskedEmail] = useState('');
  const codeRefs = useRef([]);
  const { login } = useAuth();
  const navigate = useNavigate();

  const set = (key) => (e) => setForm({ ...form, [key]: e.target.value });

  const maskEmail = (email) => {
    const [local, domain] = email.split('@');
    if (local.length <= 2) return email;
    return local[0] + '***' + local[local.length - 1] + '@' + domain;
  };

  const handleSendCode = async (e) => {
    e.preventDefault();
    setError('');
    if (!form.full_name.trim()) { setError('Please enter your name'); return; }
    if (!form.email.trim()) { setError('Please enter your email'); return; }
    if (form.password.length < 6) { setError('Password must be at least 6 characters'); return; }
    setLoading(true);
    try {
      await api.post('/auth/send-code', { email: form.email.trim().toLowerCase() });
      setMaskedEmail(maskEmail(form.email.trim()));
      setStep('verify');
    } catch (err) {
      setError(err.response?.data?.error || 'Failed to send verification code');
    }
    setLoading(false);
  };

  const handleCodeChange = (index, value) => {
    if (value.length > 1) value = value[value.length - 1];
    const newCode = [...code];
    newCode[index] = value;
    setCode(newCode);
    if (value && index < 3) codeRefs.current[index + 1]?.focus();
    if (value && index === 3 && newCode.every(d => d)) handleVerify(newCode.join(''));
  };

  const handleCodeKeyDown = (index, e) => {
    if (e.key === 'Backspace' && !code[index] && index > 0) codeRefs.current[index - 1]?.focus();
  };

  const handleVerify = async (codeStr) => {
    const fullCode = codeStr || code.join('');
    if (fullCode.length !== 4) { setError('Please enter the 4-digit code'); return; }
    setError('');
    setLoading(true);
    try {
      await api.post('/auth/verify-code', { email: form.email.trim().toLowerCase(), code: fullCode });
      const res = await signUp({ ...form, role: 'Renter' });
      login(res.data.token, res.data.refresh_token, res.data.user);
      navigate('/');
    } catch (err) {
      setError(err.response?.data?.error || 'Verification failed');
      setCode(['', '', '', '']);
      codeRefs.current[0]?.focus();
    }
    setLoading(false);
  };

  const handleResend = async () => {
    setResending(true);
    setError('');
    try {
      await api.post('/auth/send-code', { email: form.email.trim().toLowerCase() });
    } catch { setError('Failed to resend code'); }
    setResending(false);
  };

  return (
    <div className="min-h-screen flex">
      {/* Left — visual panel */}
      <div
        className="hidden lg:flex flex-1 relative overflow-hidden flex-col justify-end px-12 py-[60px]"
        style={{ background: 'linear-gradient(160deg, #0D1F13 0%, #060A06 50%, #0A0A0A 100%)' }}
      >
        <div
          className="absolute top-[15%] right-[15%] w-[500px] h-[500px] rounded-full pointer-events-none"
          style={{ background: 'radial-gradient(circle, rgba(34,197,94,0.08) 0%, transparent 70%)', filter: 'blur(60px)' }}
        />
        <div
          className="absolute bottom-[20%] left-[5%] w-[350px] h-[350px] rounded-full pointer-events-none"
          style={{ background: 'radial-gradient(circle, rgba(34,197,94,0.06) 0%, transparent 70%)', filter: 'blur(80px)' }}
        />
        <div
          className="absolute inset-0 opacity-[0.03] pointer-events-none"
          style={{
            backgroundImage:
              'linear-gradient(rgba(255,255,255,0.5) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.5) 1px, transparent 1px)',
            backgroundSize: '80px 80px',
          }}
        />

        <SpeedingCar />

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.2 }}
          className="relative z-[1]"
        >
          <Link to="/" className="no-underline inline-block mb-12">
            <Logo variant="full" size={32} />
          </Link>
          <h2 className="text-[52px] font-black tracking-[-2px] leading-[1.05] mb-4 max-w-[440px]">
            Start your<br />
            journey.
          </h2>
          <p className="text-white/40 text-base leading-relaxed max-w-[380px]">
            Join thousands of Nigerians who rent smarter with Qent.
          </p>

          <div className="flex flex-col gap-4 mt-12 pt-7 border-t border-white/[0.06]">
            {[
              [Shield, 'Verified hosts & renters', 'Every user goes through identity verification'],
              [Car, 'Premium vehicles', 'From everyday rides to luxury cars'],
              [Star, 'Rated community', 'Two-way reviews keep everyone accountable'],
            ].map(([Icon, title, sub]) => (
              <div key={title} className="flex gap-3.5 items-start">
                <div className="w-9 h-9 rounded-[10px] flex-shrink-0 bg-accent/[0.08] border border-accent/[0.12] flex items-center justify-center">
                  <Icon size={16} color="#22C55E" />
                </div>
                <div>
                  <div className="text-sm font-semibold mb-0.5">{title}</div>
                  <div className="text-xs text-white/35">{sub}</div>
                </div>
              </div>
            ))}
          </div>
        </motion.div>
      </div>

      {/* Right — form */}
      <div className="w-full lg:w-[520px] lg:flex-shrink-0 flex items-center justify-center px-12 py-10 bg-black lg:border-l lg:border-white/[0.06]">
        <div className="w-full max-w-[380px]">
          <AnimatePresence mode="wait">
            {step === 'details' ? (
              <motion.div
                key="details"
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -20 }}
                transition={{ duration: 0.4 }}
              >
                <h1 className="text-[30px] font-extrabold tracking-tight mb-1.5">Create account</h1>
                <p className="text-white/40 text-sm mb-9">
                  Join Qent and start renting
                </p>

                {error && (
                  <motion.div
                    initial={{ opacity: 0, y: -6 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="bg-red-500/[0.08] text-red-500 px-4 py-3 rounded-2xl text-[13px] font-medium mb-5 border border-red-500/15"
                  >
                    {String(error)}
                  </motion.div>
                )}

                <form onSubmit={handleSendCode}>
                  <Input label="Full Name" value={form.full_name} onChange={set('full_name')} placeholder="Your full name" />
                  <Input label="Email" type="email" value={form.email} onChange={set('email')} placeholder="you@example.com" />
                  <Input label="Password" type="password" value={form.password} onChange={set('password')} placeholder="At least 6 characters" />

                  <button
                    type="submit"
                    disabled={loading}
                    className={`w-full p-4 mt-2 text-black border-0 rounded-2xl text-[15px] font-bold flex items-center justify-center gap-2 transition-all duration-200 ${
                      loading ? 'bg-accent/50 cursor-not-allowed' : 'bg-accent cursor-pointer'
                    }`}
                  >
                    {loading ? 'Sending code...' : <>Continue <ArrowRight size={16} /></>}
                  </button>
                </form>

                <div className="flex items-center gap-3 my-7">
                  <div className="flex-1 h-px bg-white/[0.06]" />
                  <span className="text-xs text-white/25 font-medium">OR</span>
                  <div className="flex-1 h-px bg-white/[0.06]" />
                </div>

                <Link
                  to="/login"
                  className="flex items-center justify-center w-full p-4 rounded-2xl bg-white/[0.04] border border-white/[0.08] text-white text-[15px] font-semibold no-underline"
                >
                  Sign in instead
                </Link>
              </motion.div>
            ) : (
              <motion.div
                key="verify"
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -20 }}
                transition={{ duration: 0.4 }}
              >
                <button
                  onClick={() => { setStep('details'); setError(''); setCode(['', '', '', '']); }}
                  className="flex items-center gap-1.5 bg-transparent border-0 text-white/50 text-sm cursor-pointer mb-6 p-0"
                >
                  <ArrowLeft size={16} /> Back
                </button>

                <h1 className="text-[30px] font-extrabold tracking-tight mb-1.5">Verify email</h1>
                <p className="text-white/40 text-sm mb-9">
                  Enter the 4-digit code sent to {maskedEmail}
                </p>

                {error && (
                  <motion.div
                    initial={{ opacity: 0, y: -6 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="bg-red-500/[0.08] text-red-500 px-4 py-3 rounded-2xl text-[13px] font-medium mb-5 border border-red-500/15"
                  >
                    {String(error)}
                  </motion.div>
                )}

                <div className="flex gap-3.5 justify-center mb-9">
                  {[0, 1, 2, 3].map(i => (
                    <input
                      key={i}
                      ref={el => codeRefs.current[i] = el}
                      type="text"
                      inputMode="numeric"
                      maxLength={1}
                      value={code[i]}
                      onChange={e => handleCodeChange(i, e.target.value.replace(/\D/g, ''))}
                      onKeyDown={e => handleCodeKeyDown(i, e)}
                      autoFocus={i === 0}
                      className={`w-16 h-[72px] text-center text-[28px] font-extrabold rounded-2xl text-white outline-none transition-all duration-200 border-2 focus:border-accent/50 ${
                        code[i]
                          ? 'bg-accent/[0.05] border-accent/30'
                          : 'bg-white/[0.03] border-white/[0.08]'
                      }`}
                    />
                  ))}
                </div>

                <button
                  onClick={() => handleVerify()}
                  disabled={loading || code.some(d => !d)}
                  className={`w-full p-4 text-black border-0 rounded-2xl text-[15px] font-bold flex items-center justify-center gap-2 transition-all duration-200 ${
                    (loading || code.some(d => !d))
                      ? 'bg-accent/30 cursor-not-allowed'
                      : 'bg-accent cursor-pointer'
                  }`}
                >
                  {loading ? 'Verifying...' : <>Verify & Create Account <ArrowRight size={16} /></>}
                </button>

                <p className="text-center mt-6 text-white/30 text-[13px]">
                  Didn't get the code?{' '}
                  <button
                    onClick={handleResend}
                    disabled={resending}
                    className="bg-transparent border-0 text-accent font-semibold cursor-pointer text-[13px]"
                  >
                    {resending ? 'Sending...' : 'Resend'}
                  </button>
                </p>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}

function Input({ label, type = 'text', value, onChange, placeholder }) {
  return (
    <div className="mb-[18px]">
      <label className="text-[13px] font-semibold text-white/50 block mb-2">{label}</label>
      <input
        type={type}
        value={value}
        onChange={onChange}
        placeholder={placeholder}
        className="w-full px-4 py-[15px] border border-white/[0.08] bg-white/[0.03] rounded-2xl text-sm text-white outline-none box-border transition-all duration-200 focus:border-accent/40 focus:bg-accent/[0.03]"
      />
    </div>
  );
}

function SpeedingCar() {
  return (
    <div className="absolute top-[25%] left-0 right-0 h-[200px] pointer-events-none overflow-hidden">
      <div
        className="absolute bottom-[68px] left-0 right-0 h-px"
        style={{ background: 'linear-gradient(90deg, transparent 0%, rgba(34,197,94,0.15) 30%, rgba(34,197,94,0.15) 70%, transparent 100%)' }}
      />
      {[0, 1, 2, 3, 4, 5, 6].map(i => (
        <motion.div
          key={i}
          initial={{ x: '60%', opacity: 0 }}
          animate={{ x: '-120%', opacity: [0, 0.8, 0] }}
          transition={{ duration: 0.6, delay: i * 0.12, repeat: Infinity, repeatDelay: 0.1, ease: 'linear' }}
          className="absolute rounded-[2px]"
          style={{
            bottom: 74 + (i - 3) * 10,
            width: 80 + Math.random() * 60,
            height: 1.5,
            background: `linear-gradient(90deg, transparent, rgba(34,197,94,${0.2 + i * 0.06}))`,
          }}
        />
      ))}
      <motion.div
        initial={{ x: '-40%', opacity: 0 }}
        animate={{ x: '25%', opacity: 1 }}
        transition={{ duration: 0.8, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
        className="absolute bottom-[52px]"
      >
        <svg width="220" height="80" viewBox="0 0 220 80" fill="none">
          <path d="M35 50 L55 22 L95 12 L155 12 L185 28 L210 38 L210 55 L35 55 Z" fill="rgba(34,197,94,0.12)" stroke="rgba(34,197,94,0.3)" strokeWidth="1"/>
          <path d="M60 24 L90 14 L130 14 L130 34 L60 34 Z" fill="rgba(34,197,94,0.06)" stroke="rgba(34,197,94,0.15)" strokeWidth="0.5"/>
          <path d="M135 14 L155 14 L175 30 L135 34 Z" fill="rgba(34,197,94,0.06)" stroke="rgba(34,197,94,0.15)" strokeWidth="0.5"/>
          <circle cx="205" cy="42" r="8" fill="rgba(34,197,94,0.2)"/>
          <circle cx="205" cy="42" r="3" fill="rgba(34,197,94,0.5)"/>
          <circle cx="38" cy="48" r="3" fill="rgba(239,68,68,0.4)"/>
          <circle cx="170" cy="58" r="12" fill="#0A0A0A" stroke="rgba(255,255,255,0.15)" strokeWidth="1.5"/>
          <circle cx="170" cy="58" r="5" fill="rgba(255,255,255,0.08)"/>
          <circle cx="70" cy="58" r="12" fill="#0A0A0A" stroke="rgba(255,255,255,0.15)" strokeWidth="1.5"/>
          <circle cx="70" cy="58" r="5" fill="rgba(255,255,255,0.08)"/>
          <line x1="45" y1="55" x2="200" y2="55" stroke="rgba(34,197,94,0.2)" strokeWidth="1"/>
        </svg>
        <motion.div
          animate={{ opacity: [0.3, 0.6, 0.3] }}
          transition={{ duration: 2, repeat: Infinity }}
          className="absolute -right-20 top-[18px] w-[100px] h-[30px]"
          style={{
            background: 'linear-gradient(90deg, rgba(34,197,94,0.15), transparent)',
            borderRadius: '0 50% 50% 0',
            filter: 'blur(8px)',
          }}
        />
      </motion.div>
      {[0, 1, 2, 3, 4].map(i => (
        <motion.div
          key={`p${i}`}
          animate={{ x: [300, -100], opacity: [0, 0.7, 0] }}
          transition={{ duration: 0.5, delay: i * 0.2 + 0.5, repeat: Infinity, repeatDelay: 0.3, ease: 'linear' }}
          className="absolute bottom-[66px] w-[3px] h-[3px] rounded-full bg-accent/30"
        />
      ))}
    </div>
  );
}
