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
      className="max-w-[600px] mx-auto pt-[100px] px-6 pb-20"
    >
      <button
        onClick={() => navigate(-1)}
        className="flex items-center justify-center w-10 h-10 rounded-xl mb-5 bg-white/[0.06] border border-white/[0.08] text-white cursor-pointer"
      >
        <ArrowLeft size={18} />
      </button>

      {/* Profile card */}
      <div className="p-7 rounded-3xl bg-white/[0.04] border border-white/[0.08] mb-5">
        <div className="flex items-start gap-4 mb-5">
          <div
            className="w-[72px] h-[72px] rounded-[22px] flex-shrink-0 border-2 border-accent/25 flex items-center justify-center text-2xl font-extrabold text-accent overflow-hidden"
            style={{ background: 'linear-gradient(135deg, rgba(34,197,94,0.3), rgba(34,197,94,0.08))' }}
          >
            {user.profile_photo_url
              ? <img src={user.profile_photo_url} alt="" className="w-full h-full object-cover" />
              : initials}
          </div>

          <div className="flex-1">
            <h1 className="text-[22px] font-extrabold mt-0 mb-1 tracking-tight">{user.full_name}</h1>
            <div className="flex gap-1.5 flex-wrap">
              {isHost && (
                <span className="px-2.5 py-[3px] rounded-full bg-accent/[0.12] text-accent text-[11px] font-bold border border-accent/20">
                  Host
                </span>
              )}
              {user.is_verified ? (
                <span className="inline-flex items-center gap-1 px-2.5 py-[3px] rounded-full bg-blue-500/[0.12] text-blue-500 text-[11px] font-bold border border-blue-500/20">
                  <Shield size={9} /> Verified
                </span>
              ) : (
                <span className="px-2.5 py-[3px] rounded-full bg-amber-500/[0.12] text-amber-500 text-[11px] font-bold border border-amber-500/20">
                  Unverified
                </span>
              )}
            </div>
          </div>

          {!editing && (
            <button
              onClick={handleEditStart}
              className="w-[38px] h-[38px] rounded-xl flex-shrink-0 bg-white/[0.06] border border-white/[0.08] text-white/60 cursor-pointer flex items-center justify-center transition-all duration-200"
            >
              <Edit2 size={15} />
            </button>
          )}
        </div>

        <AnimatePresence mode="wait">
          {editing ? (
            <motion.div
              key="edit"
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              transition={{ duration: 0.25 }}
            >
              <div className="mb-3.5">
                <FormLabel>Full name</FormLabel>
                <FormInput
                  value={form.full_name}
                  onChange={e => setForm(f => ({ ...f, full_name: e.target.value }))}
                />
              </div>
              <div className="mb-4">
                <FormLabel>Phone number</FormLabel>
                <FormInput
                  value={form.phone}
                  onChange={e => setForm(f => ({ ...f, phone: e.target.value }))}
                  placeholder="+234..."
                />
              </div>
              {error && (
                <div className="bg-red-500/10 text-red-500 px-3.5 py-2.5 rounded-xl text-[13px] font-medium border border-red-500/20 mb-3.5">
                  {error}
                </div>
              )}
              <div className="flex gap-2.5">
                <button
                  onClick={handleSave}
                  disabled={saving}
                  className={`flex-1 p-3 rounded-2xl bg-accent text-black border-0 text-sm font-bold flex items-center justify-center gap-1.5 ${
                    saving ? 'opacity-70 cursor-not-allowed' : 'opacity-100 cursor-pointer'
                  }`}
                >
                  <Check size={15} /> {saving ? 'Saving…' : 'Save changes'}
                </button>
                <button
                  onClick={() => setEditing(false)}
                  className="px-[18px] py-3 rounded-2xl bg-white/[0.06] border border-white/[0.08] text-white/60 text-sm cursor-pointer flex items-center justify-center"
                >
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
              className="flex flex-col gap-2.5"
            >
              <InfoRow icon={<Mail size={15} />} label="Email" value={user.email} />
              {user.phone && <InfoRow icon={<Phone size={15} />} label="Phone" value={user.phone} />}
              <InfoRow
                icon={<User size={15} />}
                label="Member since"
                value={user.created_at ? new Date(user.created_at).toLocaleDateString('en-NG', { month: 'long', year: 'numeric' }) : '—'}
              />
            </motion.div>
          )}
        </AnimatePresence>

        {success && (
          <motion.div
            initial={{ opacity: 0, y: 6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0 }}
            className="mt-3.5 px-3.5 py-2.5 rounded-xl bg-accent/[0.12] text-accent text-[13px] font-semibold border border-accent/20 flex items-center gap-1.5"
          >
            <Check size={14} /> {success}
          </motion.div>
        )}
      </div>

      {/* Menu items */}
      <div className="flex flex-col gap-1 mb-4">
        {menuItems.map(({ icon: Icon, label, to, desc, accent }) => (
          <Link key={to} to={to} className="no-underline">
            <MenuRow icon={<Icon size={18} />} label={label} desc={desc} accent={accent} />
          </Link>
        ))}
      </div>

      {/* Logout */}
      <button
        onClick={handleLogout}
        className="w-full px-5 py-4 rounded-2xl bg-red-500/[0.08] border border-red-500/15 text-red-500 text-[15px] font-bold cursor-pointer flex items-center justify-center gap-2 transition-colors hover:bg-red-500/[0.14]"
      >
        <LogOut size={17} /> Sign out
      </button>
    </motion.div>
  );
}

function FormLabel({ children }) {
  return (
    <label className="block text-xs font-semibold text-white/50 mb-1.5 uppercase tracking-wider">
      {children}
    </label>
  );
}

function FormInput(props) {
  return (
    <input
      {...props}
      className="w-full px-3.5 py-3 rounded-xl bg-white/[0.06] border border-white/[0.08] text-white text-sm outline-none box-border transition-colors focus:border-accent/40"
    />
  );
}

function InfoRow({ icon, label, value }) {
  return (
    <div className="flex items-center gap-3 px-3.5 py-3 rounded-xl bg-white/[0.03]">
      <div className="text-white/30 flex-shrink-0">{icon}</div>
      <div className="flex-1 min-w-0">
        <div className="text-[11px] text-white/40 font-semibold uppercase tracking-wider mb-0.5">{label}</div>
        <div className="text-sm font-semibold text-white overflow-hidden text-ellipsis whitespace-nowrap">{value}</div>
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
      className={`flex items-center gap-3.5 px-4 py-3.5 rounded-2xl border transition-all duration-200 cursor-pointer ${
        hovered
          ? `bg-white/[0.06] ${accent ? 'border-accent/20' : 'border-white/10'}`
          : 'bg-white/[0.03] border-white/[0.06]'
      }`}
    >
      <div
        className={`w-10 h-10 rounded-[13px] flex-shrink-0 flex items-center justify-center border ${
          accent
            ? 'bg-accent/[0.12] text-accent border-accent/20'
            : 'bg-white/[0.06] text-white/70 border-white/[0.08]'
        }`}
      >
        {icon}
      </div>
      <div className="flex-1">
        <div className={`text-sm font-bold mb-0.5 ${accent ? 'text-accent' : 'text-white'}`}>{label}</div>
        <div className="text-xs text-white/40">{desc}</div>
      </div>
      <ChevronRight size={16} color="rgba(255,255,255,0.2)" />
    </div>
  );
}
