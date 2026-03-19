import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Car, Calendar, ChevronDown, ChevronUp, MessageSquare,
  X, Clock, CheckCircle, AlertCircle, XCircle, Ban, ArrowLeft,
} from 'lucide-react';
import { getMyBookings, bookingAction } from '../utils/api';
import { useAuth } from '../hooks/useAuth';

const TABS = ['All', 'Active', 'Completed', 'Cancelled'];

const STATUS_META = {
  pending_approval:  { label: 'Pending Approval',  color: '#F59E0B', bg: 'rgba(245,158,11,0.12)',  icon: Clock },
  ready_to_pay:      { label: 'Ready to Pay',       color: '#3B82F6', bg: 'rgba(59,130,246,0.12)', icon: AlertCircle },
  awaiting_pickup:   { label: 'Awaiting Pickup',    color: '#22C55E', bg: 'rgba(34,197,94,0.12)',  icon: Clock },
  active:            { label: 'Trip Active',         color: '#22C55E', bg: 'rgba(34,197,94,0.12)',  icon: CheckCircle },
  completed:         { label: 'Completed',           color: '#6B7280', bg: 'rgba(107,114,128,0.12)',icon: CheckCircle },
  cancelled:         { label: 'Cancelled',           color: '#EF4444', bg: 'rgba(239,68,68,0.12)', icon: XCircle },
  declined:          { label: 'Declined by Host',    color: '#EF4444', bg: 'rgba(239,68,68,0.12)', icon: Ban },
};

function belongs(booking, tab) {
  const s = booking.status;
  if (tab === 'All') return true;
  if (tab === 'Active') return ['pending_approval', 'ready_to_pay', 'awaiting_pickup', 'active'].includes(s);
  if (tab === 'Completed') return s === 'completed';
  if (tab === 'Cancelled') return s === 'cancelled' || s === 'declined';
  return true;
}

function fmtDate(d) {
  if (!d) return '—';
  return new Date(d).toLocaleDateString('en-NG', { month: 'short', day: 'numeric', year: 'numeric' });
}

function fmtMoney(n) {
  return '₦' + Number(n || 0).toLocaleString();
}

export default function Trips() {
  const { user } = useAuth();
  const navigate = useNavigate();
  const [bookings, setBookings] = useState([]);
  const [loading, setLoading] = useState(true);
  const [tab, setTab] = useState('All');
  const [expanded, setExpanded] = useState(null);
  const [acting, setActing] = useState(null);
  const [error, setError] = useState('');

  useEffect(() => {
    if (!user) { navigate('/login'); return; }
    getMyBookings()
      .then(r => setBookings(r.data || []))
      .catch(() => setError('Failed to load bookings'))
      .finally(() => setLoading(false));
  }, [user]);

  const handleAction = async (id, action) => {
    setActing(id + action);
    try {
      await bookingAction(id, action);
      const r = await getMyBookings();
      setBookings(r.data || []);
    } catch (e) {
      setError(e.response?.data?.error || 'Action failed');
    }
    setActing(null);
  };

  const filtered = bookings.filter(b => belongs(b, tab));

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      style={{ maxWidth: 900, margin: '0 auto', padding: '100px 24px 80px' }}
    >
      {/* Header */}
      <button onClick={() => navigate(-1)} style={backBtn}>
        <ArrowLeft size={18} />
      </button>

      <h1 style={{ fontSize: 32, fontWeight: 900, letterSpacing: -1, marginBottom: 6 }}>My Trips</h1>
      <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14, marginBottom: 32 }}>
        {bookings.length} booking{bookings.length !== 1 ? 's' : ''} total
      </p>

      {error && (
        <div style={errorBox}>{error}</div>
      )}

      {/* Tabs */}
      <div style={{ display: 'flex', gap: 8, marginBottom: 32, flexWrap: 'wrap' }}>
        {TABS.map(t => (
          <button key={t} onClick={() => setTab(t)} style={{
            padding: '10px 20px', borderRadius: 100, fontSize: 13, fontWeight: 600,
            background: tab === t ? 'var(--accent)' : 'rgba(255,255,255,0.06)',
            color: tab === t ? '#0A0A0A' : 'rgba(255,255,255,0.6)',
            border: tab === t ? 'none' : '1px solid rgba(255,255,255,0.08)',
            cursor: 'pointer', fontFamily: 'inherit', transition: 'all 0.2s',
          }}>
            {t}
          </button>
        ))}
      </div>

      {/* Content */}
      {loading ? (
        <div style={centerStyle}>
          <div style={spinner} />
          <p style={{ color: 'rgba(255,255,255,0.4)', marginTop: 16 }}>Loading trips…</p>
        </div>
      ) : filtered.length === 0 ? (
        <EmptyState tab={tab} navigate={navigate} />
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
          <AnimatePresence>
            {filtered.map(b => (
              <TripCard
                key={b.id}
                booking={b}
                expanded={expanded === b.id}
                onToggle={() => setExpanded(expanded === b.id ? null : b.id)}
                onAction={handleAction}
                acting={acting}
                navigate={navigate}
              />
            ))}
          </AnimatePresence>
        </div>
      )}
    </motion.div>
  );
}

// ─── Trip Card ───────────────────────────────────────────────────────────────

function TripCard({ booking, expanded, onToggle, onAction, acting, navigate }) {
  const meta = STATUS_META[booking.status] || { label: booking.status, color: '#6B7280', bg: 'rgba(107,114,128,0.12)', icon: Clock };
  const Icon = meta.icon;
  const car = booking.car || {};
  const photo = car.photos?.[0] || '';
  const name = car.make ? `${car.make} ${car.model} ${car.year}` : 'Car';
  const canCancel = ['pending_approval', 'ready_to_pay', 'awaiting_pickup'].includes(booking.status);

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      transition={{ duration: 0.35 }}
      style={{
        borderRadius: 20, overflow: 'hidden',
        background: 'rgba(255,255,255,0.04)',
        border: '1px solid rgba(255,255,255,0.08)',
      }}
    >
      {/* Main row */}
      <div
        onClick={onToggle}
        style={{ display: 'flex', gap: 16, padding: 20, cursor: 'pointer', alignItems: 'flex-start' }}
      >
        {/* Car photo */}
        <div style={{ width: 88, height: 66, borderRadius: 14, overflow: 'hidden', background: '#151515', flexShrink: 0 }}>
          {photo
            ? <img src={photo} alt={name} style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
            : <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 28, color: '#333' }}>🚗</div>
          }
        </div>

        {/* Info */}
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', gap: 12 }}>
            <h3 style={{ fontSize: 16, fontWeight: 700, margin: 0, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{name}</h3>
            <StatusBadge meta={meta} Icon={Icon} />
          </div>
          <div style={{ display: 'flex', gap: 16, marginTop: 8, color: 'rgba(255,255,255,0.5)', fontSize: 13, flexWrap: 'wrap' }}>
            <span style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
              <Calendar size={13} /> {fmtDate(booking.start_date)} – {fmtDate(booking.end_date)}
            </span>
            <span style={{ color: '#22C55E', fontWeight: 700 }}>{fmtMoney(booking.total_amount)}</span>
          </div>
        </div>

        {/* Expand toggle */}
        <div style={{ color: 'rgba(255,255,255,0.3)', flexShrink: 0, marginTop: 2 }}>
          {expanded ? <ChevronUp size={18} /> : <ChevronDown size={18} />}
        </div>
      </div>

      {/* Expanded section */}
      <AnimatePresence>
        {expanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.3 }}
            style={{ overflow: 'hidden' }}
          >
            <div style={{ padding: '0 20px 20px', borderTop: '1px solid rgba(255,255,255,0.06)', paddingTop: 16 }}>
              {/* Details grid */}
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))', gap: 12, marginBottom: 16 }}>
                <DetailItem label="Booking ID" value={`#${booking.id?.toString().slice(0, 8).toUpperCase()}`} />
                <DetailItem label="Days" value={booking.total_days || '—'} />
                <DetailItem label="Price/day" value={fmtMoney(booking.price_per_day)} />
                <DetailItem label="Total" value={fmtMoney(booking.total_amount)} accent />
                {booking.pickup_location && <DetailItem label="Pickup" value={booking.pickup_location} />}
                {booking.notes && <DetailItem label="Notes" value={booking.notes} />}
              </div>

              {/* Actions */}
              <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap', marginTop: 8 }}>
                {booking.conversation_id && (
                  <ActionBtn
                    icon={<MessageSquare size={15} />}
                    label="Message Host"
                    onClick={() => navigate(`/messages/${booking.conversation_id}`)}
                    variant="ghost"
                  />
                )}
                {canCancel && (
                  <ActionBtn
                    icon={<X size={15} />}
                    label={acting === booking.id + 'cancel' ? 'Cancelling…' : 'Cancel Booking'}
                    onClick={() => onAction(booking.id, 'cancel')}
                    variant="danger"
                    disabled={!!acting}
                  />
                )}
                {booking.status === 'completed' && !booking.review_id && (
                  <ActionBtn
                    icon={<CheckCircle size={15} />}
                    label="Leave Review"
                    onClick={() => navigate(`/cars/${booking.car_id}`)}
                    variant="accent"
                  />
                )}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}

// ─── Sub-components ──────────────────────────────────────────────────────────

function StatusBadge({ meta, Icon }) {
  return (
    <div style={{
      display: 'inline-flex', alignItems: 'center', gap: 6,
      padding: '5px 10px', borderRadius: 100,
      background: meta.bg, color: meta.color,
      fontSize: 11, fontWeight: 700, whiteSpace: 'nowrap', flexShrink: 0,
    }}>
      <Icon size={11} />
      {meta.label}
    </div>
  );
}

function DetailItem({ label, value, accent }) {
  return (
    <div style={{ padding: '12px 14px', borderRadius: 12, background: 'rgba(255,255,255,0.04)' }}>
      <div style={{ fontSize: 11, color: 'rgba(255,255,255,0.4)', fontWeight: 600, marginBottom: 4, textTransform: 'uppercase', letterSpacing: 0.5 }}>{label}</div>
      <div style={{ fontSize: 14, fontWeight: 700, color: accent ? '#22C55E' : 'white' }}>{value}</div>
    </div>
  );
}

function ActionBtn({ icon, label, onClick, variant, disabled }) {
  const styles = {
    ghost:  { bg: 'rgba(255,255,255,0.06)', color: 'white',   border: '1px solid rgba(255,255,255,0.1)' },
    danger: { bg: 'rgba(239,68,68,0.1)',    color: '#EF4444', border: '1px solid rgba(239,68,68,0.2)'  },
    accent: { bg: 'rgba(34,197,94,0.1)',    color: '#22C55E', border: '1px solid rgba(34,197,94,0.2)'  },
  };
  const s = styles[variant] || styles.ghost;
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        display: 'inline-flex', alignItems: 'center', gap: 6,
        padding: '9px 16px', borderRadius: 12,
        background: s.bg, color: s.color, border: s.border,
        fontSize: 13, fontWeight: 600, cursor: disabled ? 'not-allowed' : 'pointer',
        fontFamily: 'inherit', opacity: disabled ? 0.5 : 1, transition: 'opacity 0.2s',
      }}
    >
      {icon}{label}
    </button>
  );
}

function EmptyState({ tab, navigate }) {
  return (
    <div style={{ textAlign: 'center', padding: '80px 24px' }}>
      <div style={{ width: 72, height: 72, borderRadius: 24, background: 'rgba(255,255,255,0.04)', display: 'flex', alignItems: 'center', justifyContent: 'center', margin: '0 auto 20px' }}>
        <Car size={32} color="rgba(255,255,255,0.15)" />
      </div>
      <h3 style={{ fontSize: 20, fontWeight: 700, marginBottom: 8 }}>
        {tab === 'All' ? 'No trips yet' : `No ${tab.toLowerCase()} trips`}
      </h3>
      <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14, marginBottom: 28 }}>
        {tab === 'All' ? 'Your booked cars will show up here.' : `You have no ${tab.toLowerCase()} bookings.`}
      </p>
      <button
        onClick={() => navigate('/search')}
        style={{ padding: '12px 28px', background: 'var(--accent)', color: '#0A0A0A', border: 'none', borderRadius: 100, fontSize: 14, fontWeight: 700, cursor: 'pointer', fontFamily: 'inherit' }}
      >
        Find a car
      </button>
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

const errorBox = {
  background: 'rgba(239,68,68,0.1)', color: '#EF4444',
  padding: '12px 16px', borderRadius: 14, fontSize: 13,
  fontWeight: 500, marginBottom: 20, border: '1px solid rgba(239,68,68,0.2)',
};

const centerStyle = {
  display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', padding: '80px 24px',
};

const spinner = {
  width: 36, height: 36, borderRadius: '50%',
  border: '3px solid rgba(255,255,255,0.08)',
  borderTopColor: '#22C55E',
  animation: 'spin 0.8s linear infinite',
};
