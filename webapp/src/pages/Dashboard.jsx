import { useState, useEffect, useRef } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { motion, useInView } from 'framer-motion';
import {
  LayoutDashboard, Car, Eye, Star, TrendingUp, Wallet,
  Check, X, Clock, MapPin, ChevronRight, ArrowLeft,
  AlertCircle,
} from 'lucide-react';
import { getMyBookings, bookingAction, getWallet } from '../utils/api';
import api from '../utils/api';
import { useAuth } from '../hooks/useAuth';

function fmtMoney(n) {
  return '₦' + Number(n || 0).toLocaleString();
}

function fmtDate(d) {
  if (!d) return '—';
  return new Date(d).toLocaleDateString('en-NG', { month: 'short', day: 'numeric' });
}

export default function Dashboard() {
  const { user } = useAuth();
  const navigate = useNavigate();
  const [listings, setListings] = useState([]);
  const [pendingBookings, setPendingBookings] = useState([]);
  const [stats, setStats] = useState({ views: 0, active: 0, bookings: 0, rating: '0.0' });
  const [balance, setBalance] = useState(0);
  const [loading, setLoading] = useState(true);
  const [acting, setActing] = useState(null);
  const [error, setError] = useState('');

  const isHost = user?.role === 'Host' || user?.role === 'host' || user?.role === 'Admin' || user?.is_host;

  useEffect(() => {
    if (!user) { navigate('/login'); return; }

    const load = async () => {
      try {
        const [bookRes, walletRes] = await Promise.allSettled([
          getMyBookings(),
          getWallet(),
        ]);

        if (bookRes.status === 'fulfilled') {
          const all = bookRes.value.data || [];
          const asHost = all.filter(b => b.host_id === user.id || b.car?.host_id === user.id);
          const pending = asHost.filter(b => b.status === 'pending');
          setPendingBookings(pending);
          const bookingCount = asHost.filter(b => ['completed', 'active', 'confirmed'].includes(b.status)).length;
          setStats(s => ({ ...s, bookings: bookingCount, active: asHost.filter(b => b.status === 'active').length }));
        }

        if (walletRes.status === 'fulfilled') {
          setBalance(walletRes.value.data?.balance || 0);
        }

        try {
          const listRes = await api.get('/cars/mine');
          const cars = listRes.data || [];
          setListings(cars);
          const totalViews = cars.reduce((sum, c) => sum + (c.views || 0), 0);
          const avgRating = cars.length > 0
            ? (cars.reduce((sum, c) => sum + (c.rating || 0), 0) / cars.length).toFixed(1)
            : '0.0';
          setStats(s => ({ ...s, views: totalViews, rating: avgRating }));
        } catch {
          // endpoint might not exist yet
        }
      } catch {
        setError('Failed to load dashboard');
      }
      setLoading(false);
    };
    load();
  }, [user, isHost]);

  const handleAction = async (id, action) => {
    setActing(id + action);
    try {
      await bookingAction(id, action);
      setPendingBookings(prev => prev.filter(b => b.id !== id));
    } catch (e) {
      setError(e.response?.data?.error || 'Action failed');
    }
    setActing(null);
  };

  if (loading) {
    return (
      <div className="min-h-screen flex flex-col items-center justify-center gap-4">
        <div className="spinner" />
        <p className="text-white/40">Loading dashboard…</p>
      </div>
    );
  }

  const STATS = [
    { icon: Eye,        label: 'Total Views',     value: stats.views },
    { icon: Car,        label: 'Active Listings', value: listings.length },
    { icon: TrendingUp, label: 'Completed Trips', value: stats.bookings },
    { icon: Star,       label: 'Avg Rating',      value: stats.rating },
  ];

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      className="max-w-[1100px] mx-auto pt-[100px] px-6 pb-20"
    >
      <button
        onClick={() => navigate('/profile')}
        className="flex items-center justify-center w-10 h-10 rounded-xl mb-5 bg-white/[0.06] border border-white/[0.08] text-white cursor-pointer"
      >
        <ArrowLeft size={18} />
      </button>

      <div className="flex justify-between items-end mb-8 flex-wrap gap-3">
        <div>
          <div className="flex items-center gap-2.5 mb-1.5">
            <LayoutDashboard size={22} color="var(--accent)" />
            <h1 className="text-[30px] font-black tracking-tighter m-0">Host Dashboard</h1>
          </div>
          <p className="text-white/40 text-sm">Welcome back, {user?.full_name?.split(' ')[0]}</p>
        </div>
      </div>

      {error && (
        <div className="bg-red-500/10 text-red-500 px-4 py-3 rounded-2xl text-[13px] font-medium mb-5 border border-red-500/20">
          {error}
        </div>
      )}

      {/* Wallet balance card */}
      <FadeIn>
        <div
          className="px-8 py-7 rounded-3xl mb-6 border border-accent/20 flex justify-between items-center flex-wrap gap-4"
          style={{ background: 'linear-gradient(135deg, rgba(34,197,94,0.15) 0%, rgba(34,197,94,0.05) 100%)' }}
        >
          <div>
            <p className="text-white/50 text-[13px] font-semibold uppercase tracking-wider mb-2">Wallet Balance</p>
            <div className="text-4xl font-black text-accent tracking-tight">{fmtMoney(balance)}</div>
            <p className="text-white/40 text-[13px] mt-1">Available for withdrawal</p>
          </div>
          <Link
            to="/wallet"
            className="inline-flex items-center gap-2 px-6 py-3.5 rounded-2xl bg-accent text-black text-sm font-bold no-underline transition-transform hover:scale-[1.03]"
          >
            <Wallet size={16} /> View Wallet
          </Link>
        </div>
      </FadeIn>

      {/* Stats row */}
      <div className="grid gap-3.5 mb-8" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))' }}>
        {STATS.map(({ icon: Icon, label, value }, i) => (
          <FadeIn key={label} delay={i * 0.08}>
            <div className="px-[22px] py-5 rounded-[18px] bg-white/[0.04] border border-white/[0.07] transition-colors duration-200 hover:border-accent/20">
              <div className="w-9 h-9 rounded-[11px] bg-accent/10 border border-accent/15 flex items-center justify-center mb-3">
                <Icon size={17} color="#22C55E" />
              </div>
              <div className="text-[26px] font-black tracking-tight mb-1">{value}</div>
              <div className="text-xs text-white/40 font-semibold">{label}</div>
            </div>
          </FadeIn>
        ))}
      </div>

      {/* Pending bookings */}
      {pendingBookings.length > 0 && (
        <FadeIn>
          <div className="mb-10">
            <div className="flex items-center gap-2 mb-4">
              <AlertCircle size={18} color="#F59E0B" />
              <h2 className="text-xl font-extrabold m-0">Pending Approval</h2>
              <span className="px-2.5 py-0.5 rounded-full bg-amber-500/15 text-amber-500 text-xs font-bold">
                {pendingBookings.length}
              </span>
            </div>

            <div className="flex flex-col gap-3">
              {pendingBookings.map((b, i) => {
                const photo = b.car_photo || '';
                const name = b.car_name || 'Car';
                return (
                  <motion.div
                    key={b.id}
                    initial={{ opacity: 0, x: -12 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ delay: i * 0.07 }}
                    className="flex gap-3.5 px-[18px] py-4 rounded-[18px] bg-amber-500/[0.06] border border-amber-500/15 items-center flex-wrap"
                  >
                    <div className="w-[72px] h-[54px] rounded-xl overflow-hidden bg-[#151515] flex-shrink-0">
                      {photo
                        ? <img src={photo} alt={name} className="w-full h-full object-cover" />
                        : <div className="w-full h-full flex items-center justify-center text-2xl text-[#333]">🚗</div>}
                    </div>

                    <div className="flex-1 min-w-[160px]">
                      <div className="text-[15px] font-bold mb-1">{name}</div>
                      <div className="text-xs text-white/50 flex gap-3">
                        <span className="flex items-center gap-1">
                          <Clock size={11} /> {fmtDate(b.start_date)} – {fmtDate(b.end_date)}
                        </span>
                        <span className="text-accent font-bold">{fmtMoney(b.total_amount)}</span>
                      </div>
                      {b.renter_name && <div className="text-xs text-white/40 mt-[3px]">by {b.renter_name}</div>}
                    </div>

                    <div className="flex gap-2 flex-shrink-0">
                      <button
                        onClick={() => handleAction(b.id, 'approve')}
                        disabled={!!acting}
                        className={`flex items-center gap-1.5 px-4 py-2.5 rounded-xl bg-accent/15 text-accent border border-accent/25 text-[13px] font-bold ${
                          acting ? 'opacity-60 cursor-not-allowed' : 'cursor-pointer'
                        }`}
                      >
                        <Check size={14} />
                        {acting === b.id + 'approve' ? 'Approving…' : 'Approve'}
                      </button>
                      <button
                        onClick={() => handleAction(b.id, 'reject')}
                        disabled={!!acting}
                        className={`flex items-center gap-1.5 px-4 py-2.5 rounded-xl bg-red-500/10 text-red-500 border border-red-500/20 text-[13px] font-bold ${
                          acting ? 'opacity-60 cursor-not-allowed' : 'cursor-pointer'
                        }`}
                      >
                        <X size={14} />
                        {acting === b.id + 'reject' ? 'Declining…' : 'Decline'}
                      </button>
                    </div>
                  </motion.div>
                );
              })}
            </div>
          </div>
        </FadeIn>
      )}

      {/* My listings */}
      <FadeIn>
        <div>
          <div className="flex justify-between items-center mb-4 flex-wrap gap-2.5">
            <h2 className="text-xl font-extrabold m-0">My Listings</h2>
            <div className="flex gap-2">
              <Link
                to="/trips"
                className="inline-flex items-center gap-1.5 px-4 py-2.5 rounded-xl bg-white/[0.06] border border-white/[0.08] text-white/70 text-[13px] font-semibold no-underline"
              >
                All trips <ChevronRight size={14} />
              </Link>
            </div>
          </div>

          {listings.length === 0 ? (
            <div className="px-6 py-12 text-center rounded-[20px] border border-dashed border-white/10">
              <Car size={36} color="rgba(255,255,255,0.15)" className="mx-auto mb-3" />
              <p className="text-white/40 text-sm mb-5">
                You haven't listed any cars yet.
              </p>
            </div>
          ) : (
            <div className="grid gap-4" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))' }}>
              {listings.map((car, i) => (
                <ListingCard key={car.id} car={car} index={i} navigate={navigate} />
              ))}
            </div>
          )}
        </div>
      </FadeIn>
    </motion.div>
  );
}

function ListingCard({ car, index, navigate }) {
  const [hovered, setHovered] = useState(false);
  const photo = car.photos?.[0] || '';
  const name = `${car.make} ${car.model} ${car.year}`;
  const isAvailable = car.is_available !== false;

  return (
    <motion.div
      initial={{ opacity: 0, y: 14 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.07 }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={() => navigate(`/cars/${car.id}`)}
      className={`rounded-[18px] overflow-hidden cursor-pointer bg-white/[0.04] border transition-all duration-200 ${
        hovered ? 'border-accent/20 -translate-y-[3px]' : 'border-white/[0.07]'
      }`}
    >
      <div className="h-[150px] bg-[#151515] overflow-hidden">
        {photo
          ? <img src={photo} alt={name} className={`w-full h-full object-cover transition-transform duration-[400ms] ${hovered ? 'scale-105' : 'scale-100'}`} />
          : <div className="w-full h-full flex items-center justify-center text-[36px] text-[#333]">🚗</div>}
      </div>
      <div className="px-3.5 py-3">
        <div className="text-sm font-bold mb-1">{name}</div>
        <div className="flex justify-between items-center">
          <span className="text-[15px] font-extrabold text-accent">
            ₦{Number(car.price_per_day || 0).toLocaleString()}<span className="text-[11px] text-white/40 font-medium">/day</span>
          </span>
          <span
            className={`px-2.5 py-[3px] rounded-lg text-[11px] font-bold ${
              isAvailable ? 'bg-accent/10 text-accent' : 'bg-red-500/10 text-red-500'
            }`}
          >
            {isAvailable ? 'Available' : 'Unavailable'}
          </span>
        </div>
        {car.location && (
          <div className="flex items-center gap-1 text-white/40 text-xs mt-1.5">
            <MapPin size={11} /> {car.location}
          </div>
        )}
      </div>
    </motion.div>
  );
}

function FadeIn({ children, delay = 0 }) {
  const ref = useRef(null);
  const inView = useInView(ref, { once: true, margin: '-40px' });
  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 24 }}
      animate={inView ? { opacity: 1, y: 0 } : {}}
      transition={{ duration: 0.5, delay, ease: 'easeOut' }}
    >
      {children}
    </motion.div>
  );
}
