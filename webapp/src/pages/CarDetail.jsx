import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { motion } from 'framer-motion';
import { Star, MapPin, Users, Shield, ChevronLeft, ChevronRight } from 'lucide-react';
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

  if (loading) return <div className="min-h-screen flex items-center justify-center text-gray-500">Loading...</div>;
  if (!car) return null;

  const photos = car.photos || [];
  const name = `${car.make} ${car.model} ${car.year}`;
  const price = Number(car.price_per_day).toLocaleString();

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.4 }}
      className="max-w-[1100px] mx-auto pt-[100px] px-8 pb-20"
    >
      <button
        onClick={() => navigate(-1)}
        className="bg-white/[0.06] border border-white/[0.08] rounded-2xl w-11 h-11 cursor-pointer flex items-center justify-center text-white mb-6"
      >
        <ChevronLeft size={20} />
      </button>

      <div className="grid grid-cols-1 lg:grid-cols-[1.3fr_1fr] gap-10">
        {/* Left — Gallery */}
        <div>
          <div className="relative rounded-[28px] overflow-hidden bg-[#151515] h-[440px]">
            {photos.length > 0 ? (
              <motion.img
                key={photoIndex}
                initial={{ opacity: 0.5, scale: 1.02 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 0.4 }}
                src={photos[photoIndex]}
                alt={name}
                className="w-full h-full object-cover"
              />
            ) : (
              <div className="w-full h-full flex items-center justify-center text-[60px] text-[#333]">🚗</div>
            )}
            {photos.length > 1 && (
              <>
                <NavArrow dir="left" onClick={() => setPhotoIndex(i => (i - 1 + photos.length) % photos.length)} />
                <NavArrow dir="right" onClick={() => setPhotoIndex(i => (i + 1) % photos.length)} />
              </>
            )}
            <div className="absolute bottom-0 left-0 right-0 h-[100px] bg-gradient-to-b from-transparent to-black/50" />
          </div>

          {photos.length > 1 && (
            <div className="flex gap-2 mt-3">
              {photos.map((p, i) => (
                <img
                  key={i}
                  src={p}
                  alt=""
                  onClick={() => setPhotoIndex(i)}
                  className={`w-20 h-14 rounded-2xl object-cover cursor-pointer transition-all duration-200 border-2 ${
                    i === photoIndex ? 'border-accent opacity-100' : 'border-white/10 opacity-50'
                  }`}
                />
              ))}
            </div>
          )}
        </div>

        {/* Right — Details */}
        <div>
          <h1 className="text-[32px] font-black tracking-tighter mb-3">{name}</h1>

          <div className="flex items-center gap-4 flex-wrap">
            <div className="flex items-center gap-1 text-gray-500 text-sm">
              <MapPin size={14} /> {car.location}
            </div>
            {car.rating > 0 && (
              <div className="flex items-center gap-1">
                <Star size={14} fill="#FFC107" color="#FFC107" />
                <span className="text-sm font-bold">{car.rating?.toFixed(1)}</span>
                <span className="text-gray-500 text-[13px]">({car.trip_count} trips)</span>
              </div>
            )}
          </div>

          {/* Price card */}
          <div className="mt-7 p-7 rounded-3xl bg-white/[0.04] border border-white/[0.08]">
            <div className="flex justify-between items-baseline">
              <span className="text-4xl font-black text-accent">₦{price}</span>
              <span className="text-gray-500 text-sm">/day</span>
            </div>
            <button
              onClick={() => user ? navigate(`/book/${car.id}`) : navigate('/login')}
              className="w-full p-4 bg-accent text-black border-0 rounded-2xl text-base font-bold cursor-pointer mt-5 transition-transform hover:scale-[1.02]"
            >
              Book Now
            </button>
          </div>

          {/* Features */}
          <div className="mt-7">
            <h3 className="text-base font-bold mb-3.5">Features</h3>
            <div className="flex flex-wrap gap-2">
              <Tag><Users size={13} /> {car.seats || 5} seats</Tag>
              <Tag>{car.color}</Tag>
              {car.features?.map((f, i) => <Tag key={i}>{f}</Tag>)}
            </div>
          </div>

          {/* Description */}
          {car.description && (
            <div className="mt-7">
              <h3 className="text-base font-bold mb-2">About this car</h3>
              <p className="text-gray-500 text-sm leading-relaxed">{car.description}</p>
            </div>
          )}

          {/* Host */}
          <div className="mt-7 p-5 rounded-[20px] bg-white/[0.04] border border-white/[0.08] flex items-center gap-3.5">
            <div className="w-[52px] h-[52px] rounded-2xl bg-accent/10 flex items-center justify-center font-extrabold text-xl text-accent">
              {car.host_name?.[0] || '?'}
            </div>
            <div className="flex-1">
              <div className="font-bold text-[15px]">{car.host_name || 'Host'}</div>
              {hostRating && (
                <div className="text-gray-500 text-xs mt-0.5">
                  {hostRating.average_rating?.toFixed(1)} rating • {hostRating.total_reviews} reviews
                </div>
              )}
            </div>
            <div className="flex items-center gap-1 text-accent text-[13px] font-semibold">
              <Shield size={14} /> Verified
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );
}

function NavArrow({ dir, onClick }) {
  const sideClass = dir === 'left' ? 'left-4' : 'right-4';
  return (
    <button
      onClick={onClick}
      className={`absolute top-1/2 -translate-y-1/2 ${sideClass} w-11 h-11 rounded-full bg-black/50 backdrop-blur-md border border-white/10 cursor-pointer flex items-center justify-center text-white`}
    >
      {dir === 'left' ? <ChevronLeft size={20} /> : <ChevronRight size={20} />}
    </button>
  );
}

function Tag({ children }) {
  return (
    <span className="inline-flex items-center gap-1.5 bg-white/[0.06] border border-white/[0.08] px-3.5 py-2 rounded-xl text-[13px] font-medium text-gray-400">
      {children}
    </span>
  );
}
