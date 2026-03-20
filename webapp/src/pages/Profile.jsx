import { useState } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';
import {
  User, Mail, Phone, Shield, ChevronRight, LogOut,
  Car, Heart, Wallet, MessageSquare, LayoutDashboard,
  Check, X, Edit2, ArrowLeft,
} from 'lucide-react';
import { useAuth } from '../hooks/useAuth';
import api from '../utils/api';

export default function Profile() {
  const { user, logout, setUser } = useAuth();
  const navigate = useNavigate();
  const [editing, setEditing] = useState(false);
  const [form, setForm] = useState({ full_name: '', phone: '' });
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');

  if (!user) {
    navigate('/login');
    return null;
  }

  const initials = (user.full_name || 'U')
    .split(' ')
    .map(n => n[0])
    .join('')
    .toUpperCase()
    .slice(0, 2);

  const handleEditStart = () => {
    setForm({ full_name: user.full_name || '', phone: user.phone || '' });
    setEditing(true);
    setError('');
    setSuccess('');
  };

  const handleSave = async () => {
    setSaving(true);
    setError('');
    try {
      const res = await api.put('/profile', form);
      setUser(res.data);
      setEditing(false);
      setSuccess('Profile updated!');
      setTimeout(() => setSuccess(''), 3000);
    } catch (e) {
      setError(e.response?.data?.error || 'Failed to update profile');
    }
    setSaving(false);
  };

  const handleLogout = () => {
    logout();
    navigate('/');
  };

  const isHost = user.role === 'host' || user.is_host;

  const menuItems = [
    { icon: Car, label: 'My Trips', to: '/trips', desc: 'View your bookings' },
    { icon: Heart, label: 'Favorites', to: '/favorites', desc: 'Saved cars' },
    { icon: MessageSquare, label: 'Messages', to: '/messages', desc: 'Your conversations' },
    { icon: Wallet, label: 'Wallet', to: '/wallet', desc: 'Balance & transactions' },
    ...(isHost ? [{ icon: LayoutDashboard, label: 'Host Dashboard', to: '/dashboard', desc: 'Manage your listings', accent: true }] : []),
  ];

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      style={{ maxWidth: 600, margin: '0 auto', padding: '100px 24px 80px' }}
    >
      {/* Back */}
      <button onClick={() => navigate(-1)} style={backBtn}>
        <ArrowLeft size={18} />
      </button>

      {/* Profile card */}
      <div style={{
        padding: 28, borderRadius: 24,
        background: 'rgba(255,255,255,0.04)',
        border: '1px solid rgba(255,255,255,0.08)',
        marginBottom: 20,
      }}>
        <div style={{ display: 'flex', alignItems: 'flex-start', gap: 16, marginBottom: 20 }}>
          {/* Avatar */}
          <div style={{
            width: 72, height: 72, borderRadius: 22, flexShrink: 0,
            background: 'linear-gradient(135deg, rgba(34,197,94,0.3), rgba(34,197,94,0.08))',
            border: '2px solid rgba(34,197,94,0.25)',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            fontSize: 24, fontWeight: 800, color: 'var(--accent)',
            overflow: 'hidden',
          }}>
            {user.profile_photo_url
              ? <img src={user.profile_photo_url} alt="" style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
              : initials
            }
          </div>

          <div style={{ flex: 1 }}>
            <h1 style={{ fontSize: 22, fontWeight: 800, margin: '0 0 4px', letterSpacing: -0.5 }}>{user.full_name}</h1>
            <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
              {isHost && (
                <span style={{
                  padding: '3px 10px', borderRadius: 100,
                  background: 'rgba(34,197,94,0.12)', color: '#22C55E',
                  fontSize: 11, fontWeight: 700, border: '1px solid rgba(34,197,94,0.2)',
                }}>
                  Host
                </span>
              )}
              {user.is_verified && (
                <span style={{
                  display: 'inline-flex', alignItems: 'center', gap: 4,
                  padding: '3px 10px', borderRadius: 100,
                  background: 'rgba(59,130,246,0.12)', color: '#3B82F6',
                  fontSize: 11, fontWeight: 700, border: '1px solid rgba(59,130,246,0.2)',
                }}>
                  <Shield size={9} /> Verified
                </span>
              )}
              {!user.is_verified && (
                <span style={{
                  padding: '3px 10px', borderRadius: 100,
                  background: 'rgba(245,158,11,0.12)', color: '#F59E0B',
                  fontSize: 11, fontWeight: 700, border: '1px solid rgba(245,158,11,0.2)',
                }}>
                  Unverified
                </span>
              )}
            </div>
          </div>

          {/* Edit button */}
          {!editing && (
            <button onClick={handleEditStart} style={{
              width: 38, height: 38, borderRadius: 12, flexShrink: 0,
              background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
              color: 'rgba(255,255,255,0.6)', cursor: 'pointer',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              transition: 'all 0.2s',
            }}>
              <Edit2 size={15} />
            </button>
          )}
        </div>

        {/* Info / edit form */}
        <AnimatePresence mode="wait">
          {editing ? (
            <motion.div
              key="edit"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              transition={{ duration: 0.25 }}
            >
              <div style={{ marginBottom: 14 }}>
                <label style={labelStyle}>Full name</label>
                <input
                  value={form.full_name}
                  onChange={e => setForm(f => ({ ...f, full_name: e.target.value }))}
                  style={inputStyle}
                  onFocus={e => e.target.style.borderColor = 'rgba(34,197,94,0.4)'}
                  onBlur={e => e.target.style.borderColor = 'rgba(255,255,255,0.08)'}
                />
              </div>
              <div style={{ marginBottom: 16 }}>
                <label style={labelStyle}>Phone number</label>
                <input
                  value={form.phone}
                  onChange={e => setForm(f => ({ ...f, phone: e.target.value }))}
                  placeholder="+234..."
                  style={inputStyle}
                  onFocus={e => e.target.style.borderColor = 'rgba(34,197,94,0.4)'}
                  onBlur={e => e.target.style.borderColor = 'rgba(255,255,255,0.08)'}
                />
              </div>
              {error && <div style={{ ...errorBox, marginBottom: 14 }}>{error}</div>}
              <div style={{ display: 'flex', gap: 10 }}>
                <button onClick={handleSave} disabled={saving} style={{
                  flex: 1, padding: '12px', borderRadius: 14,
                  background: 'var(--accent)', color: '#0A0A0A',
                  border: 'none', fontSize: 14, fontWeight: 700,
                  cursor: saving ? 'not-allowed' : 'pointer', fontFamily: 'inherit',
                  opacity: saving ? 0.7 : 1, display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 6,
                }}>
                  <Check size={15} /> {saving ? 'Saving…' : 'Save changes'}
                </button>
                <button onClick={() => setEditing(false)} style={{
                  padding: '12px 18px', borderRadius: 14,
                  background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
                  color: 'rgba(255,255,255,0.6)', fontSize: 14, cursor: 'pointer', fontFamily: 'inherit',
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                }}>
                  <X size={15} />
                </button>
              </div>
            </motion.div>
          ) : (
            <motion.div
              key="view"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              transition={{ duration: 0.25 }}
              style={{ display: 'flex', flexDirection: 'column', gap: 10 }}
            >
              <InfoRow icon={<Mail size={15} />} label="Email" value={user.email} />
              {user.phone && <InfoRow icon={<Phone size={15} />} label="Phone" value={user.phone} />}
              <InfoRow icon={<User size={15} />} label="Member since" value={user.created_at ? new Date(user.created_at).toLocaleDateString('en-NG', { month: 'long', year: 'numeric' }) : '—'} />
            </motion.div>
          )}
        </AnimatePresence>

        {success && (
          <motion.div
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
            style={{
              marginTop: 14, padding: '10px 14px', borderRadius: 12,
              background: 'rgba(34,197,94,0.12)', color: '#22C55E',
              fontSize: 13, fontWeight: 600, border: '1px solid rgba(34,197,94,0.2)',
              display: 'flex', alignItems: 'center', gap: 6,
            }}
          >
            <Check size={14} /> {success}
          </motion.div>
        )}
      </div>

      {/* Menu items */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 4, marginBottom: 16 }}>
        {menuItems.map(({ icon: Icon, label, to, desc, accent }) => (
          <Link key={to} to={to} style={{ textDecoration: 'none' }}>
            <MenuRow icon={<Icon size={18} />} label={label} desc={desc} accent={accent} />
          </Link>
        ))}
      </div>

      {/* Logout */}
      <button onClick={handleLogout} style={{
        width: '100%', padding: '16px 20px', borderRadius: 16,
        background: 'rgba(239,68,68,0.08)', border: '1px solid rgba(239,68,68,0.15)',
        color: '#EF4444', fontSize: 15, fontWeight: 700,
        cursor: 'pointer', fontFamily: 'inherit',
        display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 8,
        transition: 'background 0.2s',
      }}
        onMouseEnter={e => e.currentTarget.style.background = 'rgba(239,68,68,0.14)'}
        onMouseLeave={e => e.currentTarget.style.background = 'rgba(239,68,68,0.08)'}
      >
        <LogOut size={17} /> Sign out
      </button>
    </motion.div>
  );
}

// ─── Sub-components ──────────────────────────────────────────────────────────

function InfoRow({ icon, label, value }) {
  return (
    <div style={{
      display: 'flex', alignItems: 'center', gap: 12,
      padding: '12px 14px', borderRadius: 12,
      background: 'rgba(255,255,255,0.03)',
    }}>
      <div style={{ color: 'rgba(255,255,255,0.3)', flexShrink: 0 }}>{icon}</div>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ fontSize: 11, color: 'rgba(255,255,255,0.4)', fontWeight: 600, textTransform: 'uppercase', letterSpacing: 0.5, marginBottom: 2 }}>{label}</div>
        <div style={{ fontSize: 14, fontWeight: 600, color: 'white', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{value}</div>
      </div>
    </div>
  );
}

function MenuRow({ icon, label, desc, accent }) {
  const [hovered, setHovered] = useState(false);
  return (
    <div
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: 'flex', alignItems: 'center', gap: 14,
        padding: '14px 16px', borderRadius: 16,
        background: hovered ? 'rgba(255,255,255,0.06)' : 'rgba(255,255,255,0.03)',
        border: `1px solid ${hovered ? (accent ? 'rgba(34,197,94,0.2)' : 'rgba(255,255,255,0.1)') : 'rgba(255,255,255,0.06)'}`,
        transition: 'all 0.2s', cursor: 'pointer',
      }}
    >
      <div style={{
        width: 40, height: 40, borderRadius: 13, flexShrink: 0,
        background: accent ? 'rgba(34,197,94,0.12)' : 'rgba(255,255,255,0.06)',
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        color: accent ? '#22C55E' : 'rgba(255,255,255,0.7)',
        border: accent ? '1px solid rgba(34,197,94,0.2)' : '1px solid rgba(255,255,255,0.08)',
      }}>
        {icon}
      </div>
      <div style={{ flex: 1 }}>
        <div style={{ fontSize: 14, fontWeight: 700, color: accent ? '#22C55E' : 'white', marginBottom: 2 }}>{label}</div>
        <div style={{ fontSize: 12, color: 'rgba(255,255,255,0.4)' }}>{desc}</div>
      </div>
      <ChevronRight size={16} color="rgba(255,255,255,0.2)" />
    </div>
  );
}

// ─── Shared styles ───────────────────────────────────────────────────────────

const backBtn = {
  display: 'flex', alignItems: 'center', justifyContent: 'center',
  width: 40, height: 40, borderRadius: 12, marginBottom: 20,
  background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
  color: 'white', cursor: 'pointer',
};

const labelStyle = {
  display: 'block', fontSize: 12, fontWeight: 600,
  color: 'rgba(255,255,255,0.5)', marginBottom: 6, textTransform: 'uppercase', letterSpacing: 0.5,
};

const inputStyle = {
  width: '100%', padding: '12px 14px', borderRadius: 12,
  background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
  color: 'white', fontSize: 14, outline: 'none', fontFamily: 'inherit',
  boxSizing: 'border-box', transition: 'border-color 0.2s',
};

const errorBox = {
  background: 'rgba(239,68,68,0.1)', color: '#EF4444',
  padding: '10px 14px', borderRadius: 12, fontSize: 13,
  fontWeight: 500, border: '1px solid rgba(239,68,68,0.2)',
};
