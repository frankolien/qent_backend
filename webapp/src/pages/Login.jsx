import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { motion } from 'framer-motion';
import { ArrowRight } from 'lucide-react';
import { useAuth } from '../hooks/useAuth';
import { signIn } from '../utils/api';
import Logo from '../components/Logo';

export default function Login() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const { login } = useAuth();
  const navigate = useNavigate();

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setLoading(true);
    try {
      const res = await signIn(email, password);
      login(res.data.token, res.data.refresh_token, res.data.user);
      navigate('/');
    } catch (err) {
      setError(err.response?.data?.error || 'Login failed');
    }
    setLoading(false);
  };

  return (
    <div className="min-h-screen flex">
      {/* Left — visual panel (hidden on small screens) */}
      <div
        className="hidden lg:flex flex-1 relative overflow-hidden flex-col justify-end px-12 py-[60px]"
        style={{ background: 'linear-gradient(160deg, #0D1F13 0%, #060A06 50%, #0A0A0A 100%)' }}
      >
        {/* Ambient glows */}
        <div
          className="absolute top-[10%] left-[20%] w-[500px] h-[500px] rounded-full pointer-events-none"
          style={{ background: 'radial-gradient(circle, rgba(34,197,94,0.08) 0%, transparent 70%)', filter: 'blur(60px)' }}
        />
        <div
          className="absolute bottom-[15%] right-[10%] w-[400px] h-[400px] rounded-full pointer-events-none"
          style={{ background: 'radial-gradient(circle, rgba(34,197,94,0.05) 0%, transparent 70%)', filter: 'blur(80px)' }}
        />
        {/* Grid pattern */}
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
            Drive<br />
            your way.
          </h2>
          <p className="text-white/40 text-base leading-relaxed max-w-[380px]">
            Rent premium cars from trusted hosts across Nigeria. Your next ride is one tap away.
          </p>

          <div className="flex gap-6 mt-12 pt-7 border-t border-white/[0.06]">
            {[
              ['500+', 'Cars listed'],
              ['10K+', 'Happy renters'],
              ['4.9', 'Avg rating'],
            ].map(([val, label]) => (
              <div key={label}>
                <div className="text-[22px] font-extrabold text-accent tracking-tight">{val}</div>
                <div className="text-xs text-white/35 mt-0.5">{label}</div>
              </div>
            ))}
          </div>
        </motion.div>
      </div>

      {/* Right — form */}
      <div className="w-full lg:w-[520px] lg:flex-shrink-0 flex items-center justify-center px-12 py-10 bg-black lg:border-l lg:border-white/[0.06]">
        <motion.div
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.6, delay: 0.1 }}
          className="w-full max-w-[380px]"
        >
          <h1 className="text-[30px] font-extrabold tracking-tight mb-1.5">Welcome back</h1>
          <p className="text-white/40 text-sm mb-9">
            Sign in to your Qent account
          </p>

          {error && (
            <motion.div
              initial={{ opacity: 0, y: -6 }}
              animate={{ opacity: 1, y: 0 }}
              className="bg-red-500/[0.08] text-red-500 px-4 py-3 rounded-2xl text-[13px] font-medium mb-5 border border-red-500/15"
            >
              {error}
            </motion.div>
          )}

          <form onSubmit={handleSubmit}>
            <Input label="Email" type="email" value={email} onChange={setEmail} placeholder="you@example.com" />
            <Input label="Password" type="password" value={password} onChange={setPassword} placeholder="Enter password" />

            <button
              type="submit"
              disabled={loading}
              className={`w-full p-4 mt-2 text-black border-0 rounded-2xl text-[15px] font-bold flex items-center justify-center gap-2 transition-all duration-200 ${
                loading ? 'bg-accent/50 cursor-not-allowed' : 'bg-accent cursor-pointer'
              }`}
            >
              {loading ? 'Signing in...' : <>Sign in <ArrowRight size={16} /></>}
            </button>
          </form>

          <div className="flex items-center gap-3 my-7">
            <div className="flex-1 h-px bg-white/[0.06]" />
            <span className="text-xs text-white/25 font-medium">OR</span>
            <div className="flex-1 h-px bg-white/[0.06]" />
          </div>

          <Link
            to="/signup"
            className="flex items-center justify-center gap-2 w-full p-4 rounded-2xl bg-white/[0.04] border border-white/[0.08] text-white text-[15px] font-semibold no-underline transition-all duration-200"
          >
            Create an account
          </Link>

          <p className="text-center mt-8 text-white/25 text-xs">
            By signing in, you agree to our Terms of Service and Privacy Policy.
          </p>
        </motion.div>
      </div>
    </div>
  );
}

function Input({ label, type, value, onChange, placeholder }) {
  return (
    <div className="mb-[18px]">
      <label className="text-[13px] font-semibold text-white/50 block mb-2">{label}</label>
      <input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full px-4 py-[15px] border border-white/[0.08] bg-white/[0.03] rounded-2xl text-sm text-white outline-none box-border transition-all duration-200 focus:border-accent/40 focus:bg-accent/[0.03]"
      />
    </div>
  );
}

function SpeedingCar() {
  return (
    <div className="absolute top-[28%] left-0 right-0 h-[200px] pointer-events-none overflow-hidden">
      {/* Road line */}
      <div
        className="absolute bottom-[68px] left-0 right-0 h-px"
        style={{ background: 'linear-gradient(90deg, transparent 0%, rgba(34,197,94,0.15) 30%, rgba(34,197,94,0.15) 70%, transparent 100%)' }}
      />

      {/* Speed lines behind car */}
      {[0, 1, 2, 3, 4, 5, 6].map(i => (
        <motion.div
          key={i}
          initial={{ x: '60%', opacity: 0 }}
          animate={{ x: '-120%', opacity: [0, 0.8, 0] }}
          transition={{
            duration: 0.6,
            delay: i * 0.12,
            repeat: Infinity,
            repeatDelay: 0.1,
            ease: 'linear',
          }}
          className="absolute rounded-[2px]"
          style={{
            bottom: 74 + (i - 3) * 10,
            width: 80 + Math.random() * 60,
            height: 1.5,
            background: `linear-gradient(90deg, transparent, rgba(34,197,94,${0.2 + i * 0.06}))`,
          }}
        />
      ))}

      {/* Car silhouette */}
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

        {/* Headlight beam */}
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

      {/* Road particles */}
      {[0, 1, 2, 3, 4].map(i => (
        <motion.div
          key={`p${i}`}
          animate={{ x: [300, -100], opacity: [0, 0.7, 0] }}
          transition={{
            duration: 0.5,
            delay: i * 0.2 + 0.5,
            repeat: Infinity,
            repeatDelay: 0.3,
            ease: 'linear',
          }}
          className="absolute bottom-[66px] w-[3px] h-[3px] rounded-full bg-accent/30"
        />
      ))}
    </div>
  );
}
