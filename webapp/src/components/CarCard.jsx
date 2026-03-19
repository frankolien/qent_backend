import { Link } from 'react-router-dom';
import { Star, MapPin, Heart } from 'lucide-react';
import { useState } from 'react';
import { motion } from 'framer-motion';

export default function CarCard({ car }) {
  const [hovered, setHovered] = useState(false);
  const [fav, setFav] = useState(false);
  const photo = car.photos?.[0] || '';
  const name = `${car.make} ${car.model} ${car.year}`;
  const price = Number(car.price_per_day).toLocaleString();
  const rating = car.rating?.toFixed(1) || '0.0';

  return (
    <Link to={`/cars/${car.id}`} style={{ textDecoration: 'none', color: 'inherit' }}>
      <motion.div
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        whileHover={{ y: -6 }}
        transition={{ duration: 0.3 }}
        style={{
          borderRadius: 20, overflow: 'hidden',
          background: 'rgba(255,255,255,0.04)',
          border: '1px solid rgba(255,255,255,0.06)',
          transition: 'border-color 0.3s',
          borderColor: hovered ? 'rgba(34,197,94,0.2)' : 'rgba(255,255,255,0.06)',
        }}
      >
        {/* Image */}
        <div style={{ position: 'relative', height: 200, background: '#151515', overflow: 'hidden' }}>
          {photo ? (
            <img src={photo} alt={name} style={{
              width: '100%', height: '100%', objectFit: 'cover',
              transform: hovered ? 'scale(1.05)' : 'scale(1)',
              transition: 'transform 0.5s ease',
            }} />
          ) : (
            <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#333', fontSize: 48 }}>&#128663;</div>
          )}

          {/* Gradient overlay */}
          <div style={{ position: 'absolute', bottom: 0, left: 0, right: 0, height: 80, background: 'linear-gradient(transparent, rgba(0,0,0,0.6))' }} />

          {/* Favorite */}
          <button onClick={(e) => { e.preventDefault(); e.stopPropagation(); setFav(!fav); }} style={{
            position: 'absolute', top: 12, right: 12, width: 36, height: 36, borderRadius: 12,
            background: 'rgba(0,0,0,0.4)', backdropFilter: 'blur(10px)', border: 'none', cursor: 'pointer',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
          }}>
            <Heart size={16} fill={fav ? '#FF385C' : 'none'} color={fav ? '#FF385C' : 'white'} />
          </button>

          {/* Trip count */}
          {car.trip_count > 0 && (
            <div style={{ position: 'absolute', bottom: 12, left: 12, background: 'rgba(0,0,0,0.5)', backdropFilter: 'blur(10px)', color: 'white', padding: '4px 10px', borderRadius: 8, fontSize: 11, fontWeight: 600 }}>
              {car.trip_count} trip{car.trip_count === 1 ? '' : 's'}
            </div>
          )}
        </div>

        {/* Info */}
        <div style={{ padding: '16px 18px 18px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
            <h3 style={{ fontSize: 16, fontWeight: 700, margin: 0, color: 'white' }}>{name}</h3>
            {Number(rating) > 0 && (
              <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                <Star size={13} fill="#FFC107" color="#FFC107" />
                <span style={{ fontSize: 13, fontWeight: 700, color: 'white' }}>{rating}</span>
              </div>
            )}
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: 4, marginTop: 6, color: 'var(--gray-500)', fontSize: 13 }}>
            <MapPin size={12} />
            <span>{car.location}</span>
          </div>

          <div style={{ marginTop: 14, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontSize: 18, fontWeight: 800, color: 'var(--accent)' }}>
              {"\u20A6"}{price}<span style={{ fontSize: 12, fontWeight: 500, color: 'var(--gray-500)' }}>/day</span>
            </span>
            <div style={{ display: 'flex', gap: 6 }}>
              <Tag>{car.seats || 5} seats</Tag>
              <Tag>{car.color}</Tag>
            </div>
          </div>
        </div>
      </motion.div>
    </Link>
  );
}

function Tag({ children }) {
  return <span style={{ background: 'rgba(255,255,255,0.06)', padding: '4px 10px', borderRadius: 8, fontSize: 11, fontWeight: 600, color: 'var(--gray-400)' }}>{children}</span>;
}
