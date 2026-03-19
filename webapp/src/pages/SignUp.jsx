import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { motion } from 'framer-motion';
import { useAuth } from '../hooks/useAuth';
import { signUp } from '../utils/api';

export default function SignUp() {
  const [form, setForm] = useState({ full_name: '', email: '', password: '', country: 'Nigeria' });
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const { login } = useAuth();
  const navigate = useNavigate();

  const set = (key) => (e) => setForm({ ...form, [key]: e.target.value });

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    if (form.password.length < 6) { setError('Password must be at least 6 characters'); return; }
    setLoading(true);
    try {
      const res = await signUp({ ...form, role: 'Renter' });
      login(res.data.token, res.data.refresh_token, res.data.user);
      navigate('/');
    } catch (err) {
      setError(err.response?.data?.error || err.response?.data?.errors || 'Sign up failed');
    }
    setLoading(false);
  };

  return (
    <div style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 24 }}>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        style={{ width: '100%', maxWidth: 420, padding: 40, borderRadius: 28, background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)' }}
      >
        <h1 style={{ fontSize: 28, fontWeight: 800, marginBottom: 6 }}>Create account</h1>
        <p style={{ color: 'var(--gray-500)', fontSize: 14, marginBottom: 32 }}>Join Qent and start renting</p>

        {error && <div style={{ background: 'rgba(239,68,68,0.1)', color: '#EF4444', padding: '12px 16px', borderRadius: 14, fontSize: 13, fontWeight: 500, marginBottom: 20, border: '1px solid rgba(239,68,68,0.2)' }}>{String(error)}</div>}

        <form onSubmit={handleSubmit}>
          <Input label="Full Name" value={form.full_name} onChange={set('full_name')} placeholder="Your full name" />
          <Input label="Email" type="email" value={form.email} onChange={set('email')} placeholder="you@example.com" />
          <Input label="Password" type="password" value={form.password} onChange={set('password')} placeholder="At least 6 characters" />

          <button type="submit" disabled={loading} style={{
            width: '100%', padding: 16, background: 'var(--accent)', color: 'var(--black)', border: 'none',
            borderRadius: 14, fontSize: 15, fontWeight: 700, cursor: 'pointer', fontFamily: 'inherit',
            opacity: loading ? 0.6 : 1, marginTop: 8,
          }}>
            {loading ? 'Creating account...' : 'Get started'}
          </button>
        </form>

        <p style={{ textAlign: 'center', marginTop: 24, color: 'var(--gray-500)', fontSize: 14 }}>
          Already have an account? <Link to="/login" style={{ color: 'var(--accent)', fontWeight: 600 }}>Sign in</Link>
        </p>
      </motion.div>
    </div>
  );
}

function Input({ label, type = 'text', value, onChange, placeholder }) {
  return (
    <div style={{ marginBottom: 16 }}>
      <label style={{ fontSize: 13, fontWeight: 600, color: 'var(--gray-400)', display: 'block', marginBottom: 8 }}>{label}</label>
      <input
        type={type} value={value} onChange={onChange} placeholder={placeholder}
        style={{
          width: '100%', padding: '14px 16px', border: '1px solid rgba(255,255,255,0.08)',
          background: 'rgba(255,255,255,0.04)', borderRadius: 14, fontSize: 14,
          color: 'white', outline: 'none', fontFamily: 'inherit', boxSizing: 'border-box',
        }}
        onFocus={e => e.target.style.borderColor = 'rgba(34,197,94,0.4)'}
        onBlur={e => e.target.style.borderColor = 'rgba(255,255,255,0.08)'}
      />
    </div>
  );
}
