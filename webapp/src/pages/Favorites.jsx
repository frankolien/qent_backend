import { useState, useEffect } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';
import { Heart, ArrowLeft, Search, Star, MapPin } from 'lucide-react';
import { getFavorites, toggleFavorite } from '../utils/api';
import { useAuth } from '../hooks/useAuth';

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
      className="max-w-[1200px] mx-auto pt-[100px] px-6 pb-20"
    >
      <button
        onClick={() => navigate(-1)}
        className="flex items-center justify-center w-10 h-10 rounded-xl mb-5 bg-white/[0.06] border border-white/[0.08] text-white cursor-pointer"
      >
        <ArrowLeft size={18} />
      </button>

      <div className="flex justify-between items-end mb-8 flex-wrap gap-3">
        <div>
          <h1 className="text-[32px] font-black tracking-tighter mb-1.5">Favorites</h1>
          <p className="text-white/40 text-sm">
            {favorites.length} saved car{favorites.length !== 1 ? 's' : ''}
          </p>
        </div>
        {favorites.length > 0 && (
          <button
            onClick={() => navigate('/search')}
            className="inline-flex items-center gap-1.5 px-5 py-2.5 rounded-full bg-white/[0.06] border border-white/[0.08] text-white text-[13px] font-semibold cursor-pointer"
          >
            <Search size={14} /> Discover more
          </button>
        )}
      </div>

      {error && (
        <div className="bg-red-500/10 text-red-500 px-4 py-3 rounded-2xl text-[13px] font-medium mb-5 border border-red-500/20">
          {error}
        </div>
      )}

      {loading ? (
        <div className="flex flex-col items-center justify-center px-6 py-20">
          <div className="spinner" />
          <p className="text-white/40 mt-4">Loading favorites…</p>
        </div>
      ) : favorites.length === 0 ? (
        <EmptyState navigate={navigate} />
      ) : (
        <div className="grid gap-5" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))' }}>
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
      className={`rounded-[20px] overflow-hidden bg-white/[0.04] border transition-all duration-300 ${
        hovered ? 'border-accent/20 -translate-y-1' : 'border-white/[0.06]'
      }`}
    >
      {/* Image */}
      <div className="relative h-[190px] bg-[#151515] overflow-hidden">
        {photo ? (
          <img
            src={photo}
            alt={name}
            className={`w-full h-full object-cover transition-transform duration-500 ${
              hovered ? 'scale-[1.06]' : 'scale-100'
            }`}
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center text-5xl text-[#333]">🚗</div>
        )}
        <div className="absolute bottom-0 left-0 right-0 h-20 bg-gradient-to-b from-transparent to-black/60" />

        <button
          onClick={onRemove}
          disabled={removing}
          className={`absolute top-3 right-3 w-9 h-9 rounded-xl backdrop-blur-md border border-red-500/30 flex items-center justify-center transition-all duration-200 ${
            removing ? 'bg-black/60 cursor-not-allowed' : 'bg-red-500/15 cursor-pointer'
          }`}
        >
          <Heart
            size={16}
            fill={removing ? 'rgba(255,255,255,0.3)' : '#FF385C'}
            color={removing ? 'rgba(255,255,255,0.3)' : '#FF385C'}
          />
        </button>
      </div>

      {/* Info */}
      <Link to={`/cars/${car.id}`} className="no-underline text-inherit">
        <div className="px-4 pt-3.5 pb-4">
          <div className="flex justify-between items-start mb-1.5">
            <h3 className="text-[15px] font-bold m-0 text-white flex-1 pr-2">{name}</h3>
            {rating && Number(rating) > 0 && (
              <div className="flex items-center gap-[3px]">
                <Star size={12} fill="#FFC107" color="#FFC107" />
                <span className="text-xs font-bold">{rating}</span>
              </div>
            )}
          </div>
          {car.location && (
            <div className="flex items-center gap-1 text-white/40 text-xs mb-3">
              <MapPin size={11} /> {car.location}
            </div>
          )}
          <div className="flex justify-between items-center">
            <span className="text-[17px] font-extrabold text-accent">
              ₦{price}<span className="text-[11px] font-medium text-white/40">/day</span>
            </span>
            <span className="px-3 py-[5px] rounded-lg bg-accent/[0.08] text-accent text-[11px] font-bold border border-accent/15">
              View car →
            </span>
          </div>
        </div>
      </Link>
    </motion.div>
  );
}

function EmptyState({ navigate }) {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.97 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.4 }}
      className="text-center px-6 py-20"
    >
      <div className="w-20 h-20 rounded-3xl bg-[#FF385C]/[0.08] border border-[#FF385C]/15 flex items-center justify-center mx-auto mb-6">
        <Heart size={36} color="#FF385C" />
      </div>
      <h3 className="text-[22px] font-extrabold mb-2.5">No favorites yet</h3>
      <p className="text-white/40 text-sm leading-relaxed max-w-[320px] mx-auto mb-8">
        Tap the heart on any car to save it here for quick access later.
      </p>
      <button
        onClick={() => navigate('/search')}
        className="px-8 py-3.5 bg-accent text-black border-0 rounded-full text-[15px] font-bold cursor-pointer"
      >
        Browse cars
      </button>
    </motion.div>
  );
}
