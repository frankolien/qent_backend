import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { motion } from 'framer-motion';
import { Star, MapPin, Users, Shield, ChevronLeft, ChevronRight, Heart } from 'lucide-react';
import { getCar, getUserRating } from '../utils/api';
import { useAuth } from '../hooks/useAuth';

export default function CarDetail() {
  const { id } = useParams();
  const navigate = useNavigate();
  const { user } = useAuth();
  const [car, setCar] = useState(null);
  const [loading, setLoading] = useState(true);
  const [photoIndex, setPhotoIndex] = useState(0);
  const [hostRating, setHostRating] = useState(null);

  useEffect(() => {
    getCar(id)
      .then((res) => {
        setCar(res.data);
        if (res.data.host_id) getUserRating(res.data.host_id).then(r => setHostRating(r.data)).catch(() => {});
      })
      .catch(() => navigate('/'))
      .finally(() => setLoading(false));
  }, [id]);

  if (loading) return <div style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--gray-500)' }}>Loading...</div>;
  if (!car) return null;

  const photos = car.photos || [];
  const name = `${car.make} ${car.model} ${car.year}`;
  const price = Number(car.price_per_day).toLocaleString();

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.4 }}
      style={{ maxWidth: 1100, margin: '0 auto', padding: '100px 32px 80px' }}
    >
      {/* Back */}
      <button onClick={() => navigate(-1)} style={{
        background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
        borderRadius: 14, width: 44, height: 44, cursor: 'pointer',
        display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'white', marginBottom: 24,
      }}>
        <ChevronLeft size={20} />
      </button>

      <div style={{ display: 'grid', gridTemplateColumns: '1.3fr 1fr', gap: 40 }}>
        {/* Left — Gallery */}
        <div>
          <div style={{ position: 'relative', borderRadius: 28, overflow: 'hidden', background: '#151515', height: 440 }}>
            {photos.length > 0 ? (
              <motion.img
                key={photoIndex}
                initial={{ opacity: 0.5, scale: 1.02 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 0.4 }}
                src={photos[photoIndex]} alt={name}
                style={{ width: '100%', height: '100%', objectFit: 'cover' }}
              />
            ) : (
              <div style={{ width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 60, color: '#333' }}>&#128663;</div>
            )}
            {photos.length > 1 && (
              <>
                <NavArrow dir="left" onClick={() => setPhotoIndex(i => (i - 1 + photos.length) % photos.length)} />
                <NavArrow dir="right" onClick={() => setPhotoIndex(i => (i + 1) % photos.length)} />
              </>
            )}
            <div style={{ position: 'absolute', bottom: 0, left: 0, right: 0, height: 100, background: 'linear-gradient(transparent, rgba(0,0,0,0.5))' }} />
          </div>

          {photos.length > 1 && (
            <div style={{ display: 'flex', gap: 8, marginTop: 12 }}>
              {photos.map((p, i) => (
                <img key={i} src={p} alt="" onClick={() => setPhotoIndex(i)} style={{
                  width: 80, height: 56, borderRadius: 14, objectFit: 'cover', cursor: 'pointer',
                  border: i === photoIndex ? '2px solid var(--accent)' : '2px solid rgba(255,255,255,0.1)',
                  opacity: i === photoIndex ? 1 : 0.5, transition: 'all 0.2s',
                }} />
              ))}
            </div>
          )}
        </div>

        {/* Right — Details */}
        <div>
          <h1 style={{ fontSize: 32, fontWeight: 900, letterSpacing: -1, marginBottom: 12 }}>{name}</h1>

          <div style={{ display: 'flex', alignItems: 'center', gap: 16, flexWrap: 'wrap' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 5, color: 'var(--gray-500)', fontSize: 14 }}>
              <MapPin size={14} /> {car.location}
            </div>
            {car.rating > 0 && (
              <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                <Star size={14} fill="#FFC107" color="#FFC107" />
                <span style={{ fontSize: 14, fontWeight: 700 }}>{car.rating?.toFixed(1)}</span>
                <span style={{ color: 'var(--gray-500)', fontSize: 13 }}>({car.trip_count} trips)</span>
              </div>
            )}
          </div>

          {/* Price card */}
          <div style={{
            marginTop: 28, padding: 28, borderRadius: 24,
            background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)',
          }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline' }}>
              <span style={{ fontSize: 36, fontWeight: 900, color: 'var(--accent)' }}>{"\u20A6"}{price}</span>
              <span style={{ color: 'var(--gray-500)', fontSize: 14 }}>/day</span>
            </div>
            <button onClick={() => user ? navigate(`/book/${car.id}`) : navigate('/login')} style={{
              width: '100%', padding: 16, background: 'var(--accent)', color: 'var(--black)', border: 'none',
              borderRadius: 16, fontSize: 16, fontWeight: 700, cursor: 'pointer', fontFamily: 'inherit', marginTop: 20,
              transition: 'transform 0.2s',
            }}
              onMouseEnter={e => e.target.style.transform = 'scale(1.02)'}
              onMouseLeave={e => e.target.style.transform = 'scale(1)'}
            >
              Book Now
            </button>
          </div>

          {/* Features */}
          <div style={{ marginTop: 28 }}>
            <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 14 }}>Features</h3>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
              <Tag><Users size={13} /> {car.seats || 5} seats</Tag>
              <Tag>{car.color}</Tag>
              {car.features?.map((f, i) => <Tag key={i}>{f}</Tag>)}
            </div>
          </div>

          {/* Description */}
          {car.description && (
            <div style={{ marginTop: 28 }}>
              <h3 style={{ fontSize: 16, fontWeight: 700, marginBottom: 8 }}>About this car</h3>
              <p style={{ color: 'var(--gray-500)', fontSize: 14, lineHeight: 1.7 }}>{car.description}</p>
            </div>
          )}

          {/* Host */}
          <div style={{
            marginTop: 28, padding: 20, borderRadius: 20,
            background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)',
            display: 'flex', alignItems: 'center', gap: 14,
          }}>
            <div style={{
              width: 52, height: 52, borderRadius: 16,
              background: 'rgba(34,197,94,0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center',
              fontWeight: 800, fontSize: 20, color: 'var(--accent)',
            }}>
              {car.host_name?.[0] || '?'}
            </div>
            <div style={{ flex: 1 }}>
              <div style={{ fontWeight: 700, fontSize: 15 }}>{car.host_name || 'Host'}</div>
              {hostRating && (
                <div style={{ color: 'var(--gray-500)', fontSize: 12, marginTop: 2 }}>
                  {hostRating.average_rating?.toFixed(1)} rating &bull; {hostRating.total_reviews} reviews
                </div>
              )}
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 4, color: 'var(--accent)', fontSize: 13, fontWeight: 600 }}>
              <Shield size={14} /> Verified
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );
}

function NavArrow({ dir, onClick }) {
  return (
    <button onClick={onClick} style={{
      position: 'absolute', top: '50%', transform: 'translateY(-50%)',
      [dir]: 16, width: 44, height: 44, borderRadius: '50%',
      background: 'rgba(0,0,0,0.5)', backdropFilter: 'blur(10px)', border: '1px solid rgba(255,255,255,0.1)',
      cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'white',
    }}>
      {dir === 'left' ? <ChevronLeft size={20} /> : <ChevronRight size={20} />}
    </button>
  );
}

function Tag({ children }) {
  return (
    <span style={{
      display: 'inline-flex', alignItems: 'center', gap: 5,
      background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
      padding: '8px 14px', borderRadius: 12, fontSize: 13, fontWeight: 500, color: 'var(--gray-400)',
    }}>
      {children}
    </span>
  );
}
