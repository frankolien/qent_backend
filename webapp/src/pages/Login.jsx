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
    <div style={{ minHeight: '100vh', display: 'flex' }}>
      {/* Left — visual panel */}
      <div style={{
        flex: 1, position: 'relative', overflow: 'hidden',
        display: 'flex', flexDirection: 'column', justifyContent: 'flex-end',
        padding: '60px 48px',
        background: 'linear-gradient(160deg, #0D1F13 0%, #060A06 50%, #0A0A0A 100%)',
      }}>
        {/* Ambient glow */}
        <div style={{
          position: 'absolute', top: '10%', left: '20%', width: 500, height: 500,
          borderRadius: '50%', background: 'radial-gradient(circle, rgba(34,197,94,0.08) 0%, transparent 70%)',
          pointerEvents: 'none', filter: 'blur(60px)',
        }} />
        <div style={{
          position: 'absolute', bottom: '15%', right: '10%', width: 400, height: 400,
          borderRadius: '50%', background: 'radial-gradient(circle, rgba(34,197,94,0.05) 0%, transparent 70%)',
          pointerEvents: 'none', filter: 'blur(80px)',
        }} />
        {/* Grid pattern */}
        <div style={{
          position: 'absolute', inset: 0, opacity: 0.03,
          backgroundImage: 'linear-gradient(rgba(255,255,255,0.5) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.5) 1px, transparent 1px)',
          backgroundSize: '80px 80px',
          pointerEvents: 'none',
        }} />

        {/* Animated car with speed lines */}
        <SpeedingCar />

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.2 }}
          style={{ position: 'relative', zIndex: 1 }}
        >
          <Link to="/" style={{ textDecoration: 'none', display: 'inline-block', marginBottom: 48 }}>
            <Logo variant="full" size={32} />
          </Link>
          <h2 style={{
            fontSize: 52, fontWeight: 900, letterSpacing: -2, lineHeight: 1.05,
            marginBottom: 16, maxWidth: 440,
          }}>
            Drive<br />
            your way.
          </h2>
          <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 16, lineHeight: 1.6, maxWidth: 380 }}>
            Rent premium cars from trusted hosts across Nigeria. Your next ride is one tap away.
          </p>

          <div style={{
            display: 'flex', gap: 24, marginTop: 48, paddingTop: 28,
            borderTop: '1px solid rgba(255,255,255,0.06)',
          }}>
            {[
              ['500+', 'Cars listed'],
              ['10K+', 'Happy renters'],
              ['4.9', 'Avg rating'],
            ].map(([val, label]) => (
              <div key={label}>
                <div style={{ fontSize: 22, fontWeight: 800, color: '#22C55E', letterSpacing: -0.5 }}>{val}</div>
                <div style={{ fontSize: 12, color: 'rgba(255,255,255,0.35)', marginTop: 2 }}>{label}</div>
              </div>
            ))}
          </div>
        </motion.div>
      </div>

      {/* Right — form */}
      <div style={{
        width: 520, flexShrink: 0,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        padding: '40px 48px',
        background: '#0A0A0A',
        borderLeft: '1px solid rgba(255,255,255,0.06)',
      }}>
        <motion.div
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.6, delay: 0.1 }}
          style={{ width: '100%', maxWidth: 380 }}
        >
          <h1 style={{ fontSize: 30, fontWeight: 800, letterSpacing: -0.5, marginBottom: 6 }}>Welcome back</h1>
          <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14, marginBottom: 36 }}>
            Sign in to your Qent account
          </p>

          {error && (
            <motion.div
              initial={{ opacity: 0, y: -6 }}
              animate={{ opacity: 1, y: 0 }}
              style={errorStyle}
            >
              {error}
            </motion.div>
          )}

          <form onSubmit={handleSubmit}>
            <Input label="Email" type="email" value={email} onChange={setEmail} placeholder="you@example.com" />
            <Input label="Password" type="password" value={password} onChange={setPassword} placeholder="Enter password" />

            <button type="submit" disabled={loading} style={{
              width: '100%', padding: 16, marginTop: 8,
              background: loading ? 'rgba(34,197,94,0.5)' : '#22C55E',
              color: '#0A0A0A', border: 'none', borderRadius: 14,
              fontSize: 15, fontWeight: 700, cursor: loading ? 'not-allowed' : 'pointer',
              fontFamily: 'inherit', transition: 'all 0.2s',
              display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 8,
            }}>
              {loading ? 'Signing in...' : <>Sign in <ArrowRight size={16} /></>}
            </button>
          </form>

          <div style={{
            display: 'flex', alignItems: 'center', gap: 12, margin: '28px 0',
          }}>
            <div style={{ flex: 1, height: 1, background: 'rgba(255,255,255,0.06)' }} />
            <span style={{ fontSize: 12, color: 'rgba(255,255,255,0.25)', fontWeight: 500 }}>OR</span>
            <div style={{ flex: 1, height: 1, background: 'rgba(255,255,255,0.06)' }} />
          </div>

          <Link to="/signup" style={{
            display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 8,
            width: '100%', padding: 16, borderRadius: 14,
            background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)',
            color: 'white', fontSize: 15, fontWeight: 600, textDecoration: 'none',
            transition: 'all 0.2s',
          }}>
            Create an account
          </Link>

          <p style={{ textAlign: 'center', marginTop: 32, color: 'rgba(255,255,255,0.25)', fontSize: 12 }}>
            By signing in, you agree to our Terms of Service and Privacy Policy.
          </p>
        </motion.div>
      </div>

      {/* Mobile responsive: hide left panel on small screens */}
      <style>{`
        @media (max-width: 900px) {
          div[style*="flex: 1"] { display: none !important; }
          div[style*="width: 520"] { width: 100% !important; border: none !important; }
        }
      `}</style>
    </div>
  );
}

function Input({ label, type, value, onChange, placeholder }) {
  return (
    <div style={{ marginBottom: 18 }}>
      <label style={{ fontSize: 13, fontWeight: 600, color: 'rgba(255,255,255,0.5)', display: 'block', marginBottom: 8 }}>{label}</label>
      <input
        type={type} value={value} onChange={(e) => onChange(e.target.value)} placeholder={placeholder}
        style={{
          width: '100%', padding: '15px 16px',
          border: '1px solid rgba(255,255,255,0.08)',
          background: 'rgba(255,255,255,0.03)', borderRadius: 14, fontSize: 14,
          color: 'white', outline: 'none', fontFamily: 'inherit', boxSizing: 'border-box',
          transition: 'all 0.2s',
        }}
        onFocus={e => { e.target.style.borderColor = 'rgba(34,197,94,0.4)'; e.target.style.background = 'rgba(34,197,94,0.03)'; }}
        onBlur={e => { e.target.style.borderColor = 'rgba(255,255,255,0.08)'; e.target.style.background = 'rgba(255,255,255,0.03)'; }}
      />
    </div>
  );
}

function SpeedingCar() {
  return (
    <div style={{
      position: 'absolute', top: '28%', left: 0, right: 0,
      height: 200, pointerEvents: 'none', overflow: 'hidden',
    }}>
      {/* Road line */}
      <motion.div
        style={{
          position: 'absolute', bottom: 68, left: 0, right: 0,
          height: 1, background: 'linear-gradient(90deg, transparent 0%, rgba(34,197,94,0.15) 30%, rgba(34,197,94,0.15) 70%, transparent 100%)',
        }}
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
          style={{
            position: 'absolute',
            bottom: 74 + (i - 3) * 10,
            width: 80 + Math.random() * 60,
            height: 1.5,
            borderRadius: 2,
            background: `linear-gradient(90deg, transparent, rgba(34,197,94,${0.2 + i * 0.06}))`,
          }}
        />
      ))}

      {/* Car silhouette */}
      <motion.div
        initial={{ x: '-40%', opacity: 0 }}
        animate={{ x: '25%', opacity: 1 }}
        transition={{ duration: 0.8, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
        style={{ position: 'absolute', bottom: 52 }}
      >
        <svg width="220" height="80" viewBox="0 0 220 80" fill="none">
          {/* Car body */}
          <path d="M35 50 L55 22 L95 12 L155 12 L185 28 L210 38 L210 55 L35 55 Z" fill="rgba(34,197,94,0.12)" stroke="rgba(34,197,94,0.3)" strokeWidth="1"/>
          {/* Windshield */}
          <path d="M60 24 L90 14 L130 14 L130 34 L60 34 Z" fill="rgba(34,197,94,0.06)" stroke="rgba(34,197,94,0.15)" strokeWidth="0.5"/>
          {/* Rear window */}
          <path d="M135 14 L155 14 L175 30 L135 34 Z" fill="rgba(34,197,94,0.06)" stroke="rgba(34,197,94,0.15)" strokeWidth="0.5"/>
          {/* Headlight glow */}
          <circle cx="205" cy="42" r="8" fill="rgba(34,197,94,0.2)"/>
          <circle cx="205" cy="42" r="3" fill="rgba(34,197,94,0.5)"/>
          {/* Tail light */}
          <circle cx="38" cy="48" r="3" fill="rgba(239,68,68,0.4)"/>
          {/* Front wheel */}
          <circle cx="170" cy="58" r="12" fill="#0A0A0A" stroke="rgba(255,255,255,0.15)" strokeWidth="1.5"/>
          <circle cx="170" cy="58" r="5" fill="rgba(255,255,255,0.08)"/>
          {/* Rear wheel */}
          <circle cx="70" cy="58" r="12" fill="#0A0A0A" stroke="rgba(255,255,255,0.15)" strokeWidth="1.5"/>
          <circle cx="70" cy="58" r="5" fill="rgba(255,255,255,0.08)"/>
          {/* Undercarriage line */}
          <line x1="45" y1="55" x2="200" y2="55" stroke="rgba(34,197,94,0.2)" strokeWidth="1"/>
        </svg>

        {/* Headlight beam */}
        <motion.div
          animate={{ opacity: [0.3, 0.6, 0.3] }}
          transition={{ duration: 2, repeat: Infinity }}
          style={{
            position: 'absolute', right: -80, top: 18, width: 100, height: 30,
            background: 'linear-gradient(90deg, rgba(34,197,94,0.15), transparent)',
            borderRadius: '0 50% 50% 0', filter: 'blur(8px)',
          }}
        />
      </motion.div>

      {/* Road particles */}
      {[0, 1, 2, 3, 4].map(i => (
        <motion.div
          key={`p${i}`}
          animate={{
            x: [300, -100],
            opacity: [0, 0.7, 0],
          }}
          transition={{
            duration: 0.5,
            delay: i * 0.2 + 0.5,
            repeat: Infinity,
            repeatDelay: 0.3,
            ease: 'linear',
          }}
          style={{
            position: 'absolute', bottom: 66,
            width: 3, height: 3, borderRadius: '50%',
            background: 'rgba(34,197,94,0.3)',
          }}
        />
      ))}
    </div>
  );
}

const errorStyle = {
  background: 'rgba(239,68,68,0.08)', color: '#EF4444',
  padding: '12px 16px', borderRadius: 14, fontSize: 13,
  fontWeight: 500, marginBottom: 20, border: '1px solid rgba(239,68,68,0.15)',
};
