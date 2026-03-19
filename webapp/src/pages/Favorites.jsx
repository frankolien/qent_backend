import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';
import { Heart, ArrowLeft, Search } from 'lucide-react';
import { getFavorites, toggleFavorite } from '../utils/api';
import { useAuth } from '../hooks/useAuth';
import { Link } from 'react-router-dom';
import { Star, MapPin } from 'lucide-react';

export default function Favorites() {
  const { user } = useAuth();
  const navigate = useNavigate();
  const [favorites, setFavorites] = useState([]);
  const [loading, setLoading] = useState(true);
  const [removing, setRemoving] = useState(null);
  const [error, setError] = useState('');

  useEffect(() => {
    if (!user) { navigate('/login'); return; }
    getFavorites()
      .then(r => setFavorites(r.data || []))
      .catch(() => setError('Failed to load favorites'))
      .finally(() => setLoading(false));
  }, [user]);

  const handleRemove = async (carId) => {
    setRemoving(carId);
    try {
      await toggleFavorite(carId);
      setFavorites(prev => prev.filter(f => (f.car?.id || f.id) !== carId));
    } catch {
      setError('Failed to remove favorite');
    }
    setRemoving(null);
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      style={{ maxWidth: 1200, margin: '0 auto', padding: '100px 24px 80px' }}
    >
      {/* Header */}
      <button onClick={() => navigate(-1)} style={backBtn}>
        <ArrowLeft size={18} />
      </button>

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end', marginBottom: 32, flexWrap: 'wrap', gap: 12 }}>
        <div>
          <h1 style={{ fontSize: 32, fontWeight: 900, letterSpacing: -1, marginBottom: 6 }}>Favorites</h1>
          <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14 }}>
            {favorites.length} saved car{favorites.length !== 1 ? 's' : ''}
          </p>
        </div>
        {favorites.length > 0 && (
          <button
            onClick={() => navigate('/search')}
            style={{
              display: 'inline-flex', alignItems: 'center', gap: 6,
              padding: '10px 20px', borderRadius: 100,
              background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
              color: 'white', fontSize: 13, fontWeight: 600, cursor: 'pointer', fontFamily: 'inherit',
            }}
          >
            <Search size={14} /> Discover more
          </button>
        )}
      </div>

      {error && (
        <div style={errorBox}>{error}</div>
      )}

      {loading ? (
        <div style={centerStyle}>
          <div style={spinnerStyle} />
          <p style={{ color: 'rgba(255,255,255,0.4)', marginTop: 16 }}>Loading favorites…</p>
        </div>
      ) : favorites.length === 0 ? (
        <EmptyState navigate={navigate} />
      ) : (
        <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
          gap: 20,
        }}>
          <AnimatePresence>
            {favorites.map((item, i) => {
              const car = item.car || item;
              return (
                <FavCard
                  key={car.id}
                  car={car}
                  index={i}
                  removing={removing === car.id}
                  onRemove={() => handleRemove(car.id)}
                />
              );
            })}
          </AnimatePresence>
        </div>
      )}
    </motion.div>
  );
}

// ─── Favorite Car Card ───────────────────────────────────────────────────────

function FavCard({ car, index, removing, onRemove }) {
  const [hovered, setHovered] = useState(false);
  const photo = car.photos?.[0] || '';
  const name = `${car.make} ${car.model} ${car.year}`;
  const price = Number(car.price_per_day || 0).toLocaleString();
  const rating = car.rating?.toFixed(1);

  return (
    <motion.div
      layout
      initial={{ opacity: 0, scale: 0.96 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.92 }}
      transition={{ duration: 0.35, delay: index * 0.04 }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        borderRadius: 20, overflow: 'hidden',
        background: 'rgba(255,255,255,0.04)',
        border: `1px solid ${hovered ? 'rgba(34,197,94,0.2)' : 'rgba(255,255,255,0.06)'}`,
        transition: 'border-color 0.3s, transform 0.3s',
        transform: hovered ? 'translateY(-4px)' : 'none',
      }}
    >
      {/* Image */}
      <div style={{ position: 'relative', height: 190, background: '#151515', overflow: 'hidden' }}>
        {photo ? (
          <img
            src={photo} alt={name}
            style={{ width: '100%', height: '100%', objectFit: 'cover', transition: 'transform 0.5s', transform: hovered ? 'scale(1.06)' : 'scale(1)' }}
          />
        ) : (
          <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 48, color: '#333' }}>🚗</div>
        )}
        <div style={{ position: 'absolute', bottom: 0, left: 0, right: 0, height: 80, background: 'linear-gradient(transparent, rgba(0,0,0,0.6))' }} />

        {/* Remove heart button */}
        <button
          onClick={onRemove}
          disabled={removing}
          style={{
            position: 'absolute', top: 12, right: 12,
            width: 36, height: 36, borderRadius: 12,
            background: removing ? 'rgba(0,0,0,0.6)' : 'rgba(239,68,68,0.15)',
            backdropFilter: 'blur(10px)',
            border: '1px solid rgba(239,68,68,0.3)',
            cursor: removing ? 'not-allowed' : 'pointer',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            transition: 'all 0.2s',
          }}
        >
          <Heart size={16} fill={removing ? 'rgba(255,255,255,0.3)' : '#FF385C'} color={removing ? 'rgba(255,255,255,0.3)' : '#FF385C'} />
        </button>
      </div>

      {/* Info */}
      <Link to={`/cars/${car.id}`} style={{ textDecoration: 'none', color: 'inherit' }}>
        <div style={{ padding: '14px 16px 16px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 6 }}>
            <h3 style={{ fontSize: 15, fontWeight: 700, margin: 0, color: 'white', flex: 1, paddingRight: 8 }}>{name}</h3>
            {rating && Number(rating) > 0 && (
              <div style={{ display: 'flex', alignItems: 'center', gap: 3 }}>
                <Star size={12} fill="#FFC107" color="#FFC107" />
                <span style={{ fontSize: 12, fontWeight: 700 }}>{rating}</span>
              </div>
            )}
          </div>
          {car.location && (
            <div style={{ display: 'flex', alignItems: 'center', gap: 4, color: 'rgba(255,255,255,0.4)', fontSize: 12, marginBottom: 12 }}>
              <MapPin size={11} /> {car.location}
            </div>
          )}
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontSize: 17, fontWeight: 800, color: 'var(--accent)' }}>
              ₦{price}<span style={{ fontSize: 11, fontWeight: 500, color: 'rgba(255,255,255,0.4)' }}>/day</span>
            </span>
            <span style={{
              padding: '5px 12px', borderRadius: 8,
              background: 'rgba(34,197,94,0.08)', color: 'var(--accent)',
              fontSize: 11, fontWeight: 700, border: '1px solid rgba(34,197,94,0.15)',
            }}>
              View car →
            </span>
          </div>
        </div>
      </Link>
    </motion.div>
  );
}

// ─── Empty state ─────────────────────────────────────────────────────────────

function EmptyState({ navigate }) {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.97 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.4 }}
      style={{ textAlign: 'center', padding: '80px 24px' }}
    >
      <div style={{
        width: 80, height: 80, borderRadius: 24,
        background: 'rgba(255,56,92,0.08)', border: '1px solid rgba(255,56,92,0.15)',
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        margin: '0 auto 24px',
      }}>
        <Heart size={36} color="#FF385C" />
      </div>
      <h3 style={{ fontSize: 22, fontWeight: 800, marginBottom: 10 }}>No favorites yet</h3>
      <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14, lineHeight: 1.6, maxWidth: 320, margin: '0 auto 32px' }}>
        Tap the heart on any car to save it here for quick access later.
      </p>
      <button
        onClick={() => navigate('/search')}
        style={{
          padding: '14px 32px', background: 'var(--accent)', color: '#0A0A0A',
          border: 'none', borderRadius: 100, fontSize: 15, fontWeight: 700,
          cursor: 'pointer', fontFamily: 'inherit',
        }}
      >
        Browse cars
      </button>
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

const centerStyle = {
  display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', padding: '80px 24px',
};

const spinnerStyle = {
  width: 36, height: 36, borderRadius: '50%',
  border: '3px solid rgba(255,255,255,0.08)',
  borderTopColor: '#22C55E',
  animation: 'spin 0.8s linear infinite',
};
