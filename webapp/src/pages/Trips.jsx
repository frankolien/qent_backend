import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';
import {
  Car, Calendar, ChevronDown, ChevronUp, MessageSquare,
  X, Clock, CheckCircle, AlertCircle, XCircle, Ban, ArrowLeft,
} from 'lucide-react';
import api, { getMyBookings, bookingAction } from '../utils/api';
import { useAuth } from '../hooks/useAuth';

const TABS = ['All', 'Active', 'Completed', 'Cancelled'];

const STATUS_META = {
  pending:    { label: 'Pending Approval',  color: '#F59E0B', bg: 'rgba(245,158,11,0.12)',  icon: Clock },
  approved:   { label: 'Ready to Pay',      color: '#3B82F6', bg: 'rgba(59,130,246,0.12)',  icon: AlertCircle },
  confirmed:  { label: 'Awaiting Pickup',   color: '#22C55E', bg: 'rgba(34,197,94,0.12)',   icon: Clock },
  active:     { label: 'Trip Active',       color: '#22C55E', bg: 'rgba(34,197,94,0.12)',   icon: CheckCircle },
  completed:  { label: 'Completed',         color: '#6B7280', bg: 'rgba(107,114,128,0.12)', icon: CheckCircle },
  cancelled:  { label: 'Cancelled',         color: '#EF4444', bg: 'rgba(239,68,68,0.12)',   icon: XCircle },
  rejected:   { label: 'Declined by Host',  color: '#EF4444', bg: 'rgba(239,68,68,0.12)',   icon: Ban },
};

function belongs(booking, tab) {
  const s = booking.status;
  if (tab === 'All') return true;
  if (tab === 'Active') return ['pending', 'approved', 'confirmed', 'active'].includes(s);
  if (tab === 'Completed') return s === 'completed';
  if (tab === 'Cancelled') return s === 'cancelled' || s === 'rejected';
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

  const handlePay = async (bookingId) => {
    setActing(bookingId + 'pay');
    try {
      const res = await api.post('/payments/initiate', { booking_id: bookingId });
      const url = res.data?.authorization_url;
      if (url) {
        window.location.href = url;
      } else {
        setError('Could not get payment link. Please try again.');
      }
    } catch (e) {
      setError(e.response?.data?.error || 'Payment failed');
    }
    setActing(null);
  };

  const filtered = bookings.filter(b => belongs(b, tab));

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      className="max-w-[900px] mx-auto pt-[100px] px-6 pb-20"
    >
      <button
        onClick={() => navigate(-1)}
        className="flex items-center justify-center w-10 h-10 rounded-xl mb-5 bg-white/[0.06] border border-white/[0.08] text-white cursor-pointer"
      >
        <ArrowLeft size={18} />
      </button>

      <h1 className="text-[32px] font-black tracking-tighter mb-1.5">My Trips</h1>
      <p className="text-white/40 text-sm mb-8">
        {bookings.length} booking{bookings.length !== 1 ? 's' : ''} total
      </p>

      {error && (
        <div className="bg-red-500/10 text-red-500 px-4 py-3 rounded-2xl text-[13px] font-medium mb-5 border border-red-500/20">
          {error}
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-2 mb-8 flex-wrap">
        {TABS.map(t => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-5 py-2.5 rounded-full text-[13px] font-semibold cursor-pointer transition-all duration-200 ${
              tab === t
                ? 'bg-accent text-black border-0'
                : 'bg-white/[0.06] text-white/60 border border-white/[0.08]'
            }`}
          >
            {t}
          </button>
        ))}
      </div>

      {loading ? (
        <div className="flex flex-col items-center justify-center px-6 py-20">
          <div className="spinner" />
          <p className="text-white/40 mt-4">Loading trips…</p>
        </div>
      ) : filtered.length === 0 ? (
        <EmptyState tab={tab} navigate={navigate} />
      ) : (
        <div className="flex flex-col gap-4">
          <AnimatePresence>
            {filtered.map(b => (
              <TripCard
                key={b.id}
                booking={b}
                expanded={expanded === b.id}
                onToggle={() => setExpanded(expanded === b.id ? null : b.id)}
                onAction={handleAction}
                onPay={handlePay}
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

function TripCard({ booking, expanded, onToggle, onAction, onPay, acting, navigate }) {
  const meta = STATUS_META[booking.status] || { label: booking.status, color: '#6B7280', bg: 'rgba(107,114,128,0.12)', icon: Clock };
  const Icon = meta.icon;
  const photo = booking.car_photo || '';
  const name = booking.car_name || 'Car';
  const canCancel = ['pending', 'approved', 'confirmed'].includes(booking.status);
  const canPay = booking.status === 'approved';

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      transition={{ duration: 0.35 }}
      className="rounded-[20px] overflow-hidden bg-white/[0.04] border border-white/[0.08]"
    >
      {/* Main row */}
      <div
        onClick={onToggle}
        className="flex gap-4 p-5 cursor-pointer items-start"
      >
        {/* Car photo */}
        <div className="w-[88px] h-[66px] rounded-2xl overflow-hidden bg-[#151515] flex-shrink-0">
          {photo
            ? <img src={photo} alt={name} className="w-full h-full object-cover" />
            : <div className="w-full h-full flex items-center justify-center text-[28px] text-[#333]">🚗</div>
          }
        </div>

        {/* Info */}
        <div className="flex-1 min-w-0">
          <div className="flex justify-between items-start gap-3">
            <h3 className="text-base font-bold m-0 whitespace-nowrap overflow-hidden text-ellipsis">{name}</h3>
            <StatusBadge meta={meta} Icon={Icon} />
          </div>
          <div className="flex gap-4 mt-2 text-white/50 text-[13px] flex-wrap">
            <span className="flex items-center gap-1">
              <Calendar size={13} /> {fmtDate(booking.start_date)} – {fmtDate(booking.end_date)}
            </span>
            <span className="text-accent font-bold">{fmtMoney(booking.total_amount)}</span>
          </div>
        </div>

        <div className="text-white/30 flex-shrink-0 mt-0.5">
          {expanded ? <ChevronUp size={18} /> : <ChevronDown size={18} />}
        </div>
      </div>

      <AnimatePresence>
        {expanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.3 }}
            className="overflow-hidden"
          >
            <div className="px-5 pb-5 pt-4 border-t border-white/[0.06]">
              <div
                className="grid gap-3 mb-4"
                style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))' }}
              >
                <DetailItem label="Booking ID" value={`#${booking.id?.toString().slice(0, 8).toUpperCase()}`} />
                <DetailItem label="Days" value={booking.total_days || '—'} />
                <DetailItem label="Price/day" value={fmtMoney(booking.price_per_day)} />
                <DetailItem label="Total" value={fmtMoney(booking.total_amount)} accent />
                {booking.pickup_location && <DetailItem label="Pickup" value={booking.pickup_location} />}
                {booking.notes && <DetailItem label="Notes" value={booking.notes} />}
              </div>

              <div className="flex gap-2.5 flex-wrap mt-2">
                {canPay && (
                  <ActionBtn
                    icon={<CheckCircle size={15} />}
                    label={acting === booking.id + 'pay' ? 'Processing…' : 'Pay Now'}
                    onClick={() => onPay(booking.id)}
                    variant="primary"
                    disabled={!!acting}
                  />
                )}
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
                    onClick={() => navigate(`/review?booking=${booking.id}&reviewee=${booking.host_id}&car=${encodeURIComponent(name)}`)}
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

function StatusBadge({ meta, Icon }) {
  return (
    <div
      className="inline-flex items-center gap-1.5 px-2.5 py-[5px] rounded-full text-[11px] font-bold whitespace-nowrap flex-shrink-0"
      style={{ background: meta.bg, color: meta.color }}
    >
      <Icon size={11} />
      {meta.label}
    </div>
  );
}

function DetailItem({ label, value, accent }) {
  return (
    <div className="px-3.5 py-3 rounded-xl bg-white/[0.04]">
      <div className="text-[11px] text-white/40 font-semibold mb-1 uppercase tracking-wider">{label}</div>
      <div className={`text-sm font-bold ${accent ? 'text-accent' : 'text-white'}`}>{value}</div>
    </div>
  );
}

function ActionBtn({ icon, label, onClick, variant, disabled }) {
  const variants = {
    primary: 'bg-accent text-black border-0',
    ghost: 'bg-white/[0.06] text-white border border-white/10',
    danger: 'bg-red-500/10 text-red-500 border border-red-500/20',
    accent: 'bg-accent/10 text-accent border border-accent/20',
  };
  const cls = variants[variant] || variants.ghost;
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`inline-flex items-center gap-1.5 px-4 py-2.5 rounded-xl text-[13px] font-semibold transition-opacity ${cls} ${
        disabled ? 'opacity-50 cursor-not-allowed' : 'opacity-100 cursor-pointer'
      }`}
    >
      {icon}{label}
    </button>
  );
}

function EmptyState({ tab, navigate }) {
  return (
    <div className="text-center px-6 py-20">
      <div className="w-[72px] h-[72px] rounded-3xl bg-white/[0.04] flex items-center justify-center mx-auto mb-5">
        <Car size={32} color="rgba(255,255,255,0.15)" />
      </div>
      <h3 className="text-xl font-bold mb-2">
        {tab === 'All' ? 'No trips yet' : `No ${tab.toLowerCase()} trips`}
      </h3>
      <p className="text-white/40 text-sm mb-7">
        {tab === 'All' ? 'Your booked cars will show up here.' : `You have no ${tab.toLowerCase()} bookings.`}
      </p>
      <button
        onClick={() => navigate('/search')}
        className="px-7 py-3 bg-accent text-black border-0 rounded-full text-sm font-bold cursor-pointer"
      >
        Find a car
      </button>
    </div>
  );
}
