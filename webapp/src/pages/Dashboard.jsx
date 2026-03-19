import { useState, useEffect, useRef } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { motion, useInView } from 'framer-motion';
import {
  LayoutDashboard, Car, Eye, Star, TrendingUp, Wallet,
  Check, X, Clock, MapPin, ChevronRight, ArrowLeft, Plus,
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
    // Non-hosts can still view dashboard but will see empty state

    const load = async () => {
      try {
        const [bookRes, walletRes] = await Promise.allSettled([
          getMyBookings(),
          getWallet(),
        ]);

        if (bookRes.status === 'fulfilled') {
          const all = bookRes.value.data || [];
          // Filter bookings where user is host
          const asHost = all.filter(b => b.host_id === user.id || b.car?.host_id === user.id);
          const pending = asHost.filter(b => b.status === 'pending');
          setPendingBookings(pending);
          const bookingCount = asHost.filter(b => ['completed', 'active', 'confirmed'].includes(b.status)).length;
          setStats(s => ({ ...s, bookings: bookingCount, active: asHost.filter(b => b.status === 'active').length }));
        }

        if (walletRes.status === 'fulfilled') {
          setBalance(walletRes.value.data?.balance || 0);
        }

        // Fetch host listings
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
      <div style={{ minHeight: '100vh', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: 16 }}>
        <div style={spinnerStyle} />
        <p style={{ color: 'rgba(255,255,255,0.4)' }}>Loading dashboard…</p>
      </div>
    );
  }

  const STATS = [
    { icon: Eye,         label: 'Total Views',     value: stats.views    },
    { icon: Car,         label: 'Active Listings',  value: listings.length },
    { icon: TrendingUp,  label: 'Completed Trips',  value: stats.bookings },
    { icon: Star,        label: 'Avg Rating',       value: stats.rating    },
  ];

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      style={{ maxWidth: 1100, margin: '0 auto', padding: '100px 24px 80px' }}
    >
      {/* Header */}
      <button onClick={() => navigate('/profile')} style={backBtn}>
        <ArrowLeft size={18} />
      </button>

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end', marginBottom: 32, flexWrap: 'wrap', gap: 12 }}>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 6 }}>
            <LayoutDashboard size={22} color="var(--accent)" />
            <h1 style={{ fontSize: 30, fontWeight: 900, letterSpacing: -1, margin: 0 }}>Host Dashboard</h1>
          </div>
          <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14 }}>Welcome back, {user?.full_name?.split(' ')[0]}</p>
        </div>
      </div>

      {error && <div style={errorBox}>{error}</div>}

      {/* ── Wallet balance card ─────────────────────────────────────── */}
      <FadeIn>
        <div style={{
          padding: '28px 32px', borderRadius: 24, marginBottom: 24,
          background: 'linear-gradient(135deg, rgba(34,197,94,0.15) 0%, rgba(34,197,94,0.05) 100%)',
          border: '1px solid rgba(34,197,94,0.2)',
          display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: 16,
        }}>
          <div>
            <p style={{ color: 'rgba(255,255,255,0.5)', fontSize: 13, fontWeight: 600, textTransform: 'uppercase', letterSpacing: 1, marginBottom: 8 }}>Wallet Balance</p>
            <div style={{ fontSize: 40, fontWeight: 900, color: '#22C55E', letterSpacing: -1 }}>{fmtMoney(balance)}</div>
            <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 13, marginTop: 4 }}>Available for withdrawal</p>
          </div>
          <Link to="/wallet" style={{
            display: 'inline-flex', alignItems: 'center', gap: 8,
            padding: '14px 24px', borderRadius: 14,
            background: '#22C55E', color: '#0A0A0A',
            fontSize: 14, fontWeight: 700, textDecoration: 'none',
            transition: 'transform 0.2s',
          }}
            onMouseEnter={e => e.currentTarget.style.transform = 'scale(1.03)'}
            onMouseLeave={e => e.currentTarget.style.transform = 'scale(1)'}
          >
            <Wallet size={16} /> View Wallet
          </Link>
        </div>
      </FadeIn>

      {/* ── Stats row ─────────────────────────────────────────────── */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: 14, marginBottom: 32 }}>
        {STATS.map(({ icon: Icon, label, value }, i) => (
          <FadeIn key={label} delay={i * 0.08}>
            <div style={{
              padding: '20px 22px', borderRadius: 18,
              background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.07)',
              transition: 'border-color 0.2s',
            }}
              onMouseEnter={e => e.currentTarget.style.borderColor = 'rgba(34,197,94,0.2)'}
              onMouseLeave={e => e.currentTarget.style.borderColor = 'rgba(255,255,255,0.07)'}
            >
              <div style={{
                width: 36, height: 36, borderRadius: 11,
                background: 'rgba(34,197,94,0.1)', border: '1px solid rgba(34,197,94,0.15)',
                display: 'flex', alignItems: 'center', justifyContent: 'center', marginBottom: 12,
              }}>
                <Icon size={17} color="#22C55E" />
              </div>
              <div style={{ fontSize: 26, fontWeight: 900, letterSpacing: -0.5, marginBottom: 4 }}>{value}</div>
              <div style={{ fontSize: 12, color: 'rgba(255,255,255,0.4)', fontWeight: 600 }}>{label}</div>
            </div>
          </FadeIn>
        ))}
      </div>

      {/* ── Pending bookings ──────────────────────────────────────── */}
      {pendingBookings.length > 0 && (
        <FadeIn>
          <div style={{ marginBottom: 40 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 16 }}>
              <AlertCircle size={18} color="#F59E0B" />
              <h2 style={{ fontSize: 20, fontWeight: 800, margin: 0 }}>Pending Approval</h2>
              <span style={{
                padding: '2px 10px', borderRadius: 100,
                background: 'rgba(245,158,11,0.15)', color: '#F59E0B',
                fontSize: 12, fontWeight: 700,
              }}>{pendingBookings.length}</span>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              {pendingBookings.map((b, i) => {
                const photo = b.car_photo || '';
                const name = b.car_name || 'Car';
                return (
                  <motion.div
                    key={b.id}
                    initial={{ opacity: 0, x: -12 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ delay: i * 0.07 }}
                    style={{
                      display: 'flex', gap: 14, padding: '16px 18px', borderRadius: 18,
                      background: 'rgba(245,158,11,0.06)', border: '1px solid rgba(245,158,11,0.15)',
                      alignItems: 'center', flexWrap: 'wrap',
                    }}
                  >
                    {/* Car photo */}
                    <div style={{ width: 72, height: 54, borderRadius: 12, overflow: 'hidden', background: '#151515', flexShrink: 0 }}>
                      {photo
                        ? <img src={photo} alt={name} style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
                        : <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 24, color: '#333' }}>🚗</div>
                      }
                    </div>

                    <div style={{ flex: 1, minWidth: 160 }}>
                      <div style={{ fontSize: 15, fontWeight: 700, marginBottom: 4 }}>{name}</div>
                      <div style={{ fontSize: 12, color: 'rgba(255,255,255,0.5)', display: 'flex', gap: 12 }}>
                        <span style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                          <Clock size={11} /> {fmtDate(b.start_date)} – {fmtDate(b.end_date)}
                        </span>
                        <span style={{ color: '#22C55E', fontWeight: 700 }}>{fmtMoney(b.total_amount)}</span>
                      </div>
                      {b.renter_name && <div style={{ fontSize: 12, color: 'rgba(255,255,255,0.4)', marginTop: 3 }}>by {b.renter_name}</div>}
                    </div>

                    {/* Action buttons */}
                    <div style={{ display: 'flex', gap: 8, flexShrink: 0 }}>
                      <button
                        onClick={() => handleAction(b.id, 'approve')}
                        disabled={!!acting}
                        style={{
                          display: 'flex', alignItems: 'center', gap: 5,
                          padding: '9px 16px', borderRadius: 12,
                          background: 'rgba(34,197,94,0.15)', color: '#22C55E',
                          border: '1px solid rgba(34,197,94,0.25)', fontSize: 13, fontWeight: 700,
                          cursor: acting ? 'not-allowed' : 'pointer', fontFamily: 'inherit',
                          opacity: acting ? 0.6 : 1,
                        }}
                      >
                        <Check size={14} />
                        {acting === b.id + 'approve' ? 'Approving…' : 'Approve'}
                      </button>
                      <button
                        onClick={() => handleAction(b.id, 'reject')}
                        disabled={!!acting}
                        style={{
                          display: 'flex', alignItems: 'center', gap: 5,
                          padding: '9px 16px', borderRadius: 12,
                          background: 'rgba(239,68,68,0.1)', color: '#EF4444',
                          border: '1px solid rgba(239,68,68,0.2)', fontSize: 13, fontWeight: 700,
                          cursor: acting ? 'not-allowed' : 'pointer', fontFamily: 'inherit',
                          opacity: acting ? 0.6 : 1,
                        }}
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

      {/* ── My listings ───────────────────────────────────────────── */}
      <FadeIn>
        <div>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16, flexWrap: 'wrap', gap: 10 }}>
            <h2 style={{ fontSize: 20, fontWeight: 800, margin: 0 }}>My Listings</h2>
            <div style={{ display: 'flex', gap: 8 }}>
              <Link to="/trips" style={{
                display: 'inline-flex', alignItems: 'center', gap: 6,
                padding: '9px 16px', borderRadius: 12,
                background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
                color: 'rgba(255,255,255,0.7)', fontSize: 13, fontWeight: 600, textDecoration: 'none',
              }}>
                All trips <ChevronRight size={14} />
              </Link>
            </div>
          </div>

          {listings.length === 0 ? (
            <div style={{
              padding: '48px 24px', textAlign: 'center',
              borderRadius: 20, border: '1px dashed rgba(255,255,255,0.1)',
            }}>
              <Car size={36} color="rgba(255,255,255,0.15)" style={{ margin: '0 auto 12px' }} />
              <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14, marginBottom: 20 }}>
                You haven't listed any cars yet.
              </p>
            </div>
          ) : (
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))', gap: 16 }}>
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

// ─── Listing Card ─────────────────────────────────────────────────────────────

function ListingCard({ car, index, navigate }) {
  const [hovered, setHovered] = useState(false);
  const photo = car.photos?.[0] || '';
  const name = `${car.make} ${car.model} ${car.year}`;

  return (
    <motion.div
      initial={{ opacity: 0, y: 14 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.07 }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={() => navigate(`/cars/${car.id}`)}
      style={{
        borderRadius: 18, overflow: 'hidden', cursor: 'pointer',
        background: 'rgba(255,255,255,0.04)',
        border: `1px solid ${hovered ? 'rgba(34,197,94,0.2)' : 'rgba(255,255,255,0.07)'}`,
        transition: 'all 0.2s', transform: hovered ? 'translateY(-3px)' : 'none',
      }}
    >
      <div style={{ height: 150, background: '#151515', overflow: 'hidden' }}>
        {photo
          ? <img src={photo} alt={name} style={{ width: '100%', height: '100%', objectFit: 'cover', transition: 'transform 0.4s', transform: hovered ? 'scale(1.05)' : 'scale(1)' }} />
          : <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 36, color: '#333' }}>🚗</div>
        }
      </div>
      <div style={{ padding: '12px 14px' }}>
        <div style={{ fontSize: 14, fontWeight: 700, marginBottom: 4 }}>{name}</div>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontSize: 15, fontWeight: 800, color: '#22C55E' }}>
            ₦{Number(car.price_per_day || 0).toLocaleString()}<span style={{ fontSize: 11, color: 'rgba(255,255,255,0.4)', fontWeight: 500 }}>/day</span>
          </span>
          <span style={{
            padding: '3px 9px', borderRadius: 8, fontSize: 11, fontWeight: 700,
            background: car.is_available !== false ? 'rgba(34,197,94,0.1)' : 'rgba(239,68,68,0.1)',
            color: car.is_available !== false ? '#22C55E' : '#EF4444',
          }}>
            {car.is_available !== false ? 'Available' : 'Unavailable'}
          </span>
        </div>
        {car.location && (
          <div style={{ display: 'flex', alignItems: 'center', gap: 4, color: 'rgba(255,255,255,0.4)', fontSize: 12, marginTop: 6 }}>
            <MapPin size={11} /> {car.location}
          </div>
        )}
      </div>
    </motion.div>
  );
}

// ─── FadeIn helper ────────────────────────────────────────────────────────────

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

const spinnerStyle = {
  width: 36, height: 36, borderRadius: '50%',
  border: '3px solid rgba(255,255,255,0.08)',
  borderTopColor: '#22C55E',
  animation: 'spin 0.8s linear infinite',
};
