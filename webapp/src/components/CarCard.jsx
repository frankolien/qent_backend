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
    <Link to={`/cars/${car.id}`} className="no-underline text-inherit">
      <motion.div
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        whileHover={{ y: -6 }}
        transition={{ duration: 0.3 }}
        className={`rounded-[20px] overflow-hidden bg-white/[0.04] border transition-colors duration-300 ${
          hovered ? 'border-accent/20' : 'border-white/[0.06]'
        }`}
      >
        {/* Image */}
        <div className="relative h-[200px] bg-[#151515] overflow-hidden">
          {photo ? (
            <img
              src={photo}
              alt={name}
              className={`w-full h-full object-cover transition-transform duration-500 ease-out ${
                hovered ? 'scale-105' : 'scale-100'
              }`}
            />
          ) : (
            <div className="w-full h-full flex items-center justify-center text-[#333] text-5xl">🚗</div>
          )}

          {/* Gradient overlay */}
          <div className="absolute bottom-0 left-0 right-0 h-20 bg-gradient-to-b from-transparent to-black/60" />

          {/* Favorite */}
          <button
            onClick={(e) => { e.preventDefault(); e.stopPropagation(); setFav(!fav); }}
            className="absolute top-3 right-3 w-9 h-9 rounded-xl bg-black/40 backdrop-blur-md border-0 cursor-pointer flex items-center justify-center"
          >
            <Heart size={16} fill={fav ? '#FF385C' : 'none'} color={fav ? '#FF385C' : 'white'} />
          </button>

          {/* Trip count */}
          {car.trip_count > 0 && (
            <div className="absolute bottom-3 left-3 bg-black/50 backdrop-blur-md text-white px-2.5 py-1 rounded-lg text-[11px] font-semibold">
              {car.trip_count} trip{car.trip_count === 1 ? '' : 's'}
            </div>
          )}
        </div>

        {/* Info */}
        <div className="px-[18px] pt-4 pb-[18px]">
          <div className="flex justify-between items-start">
            <h3 className="text-base font-bold m-0 text-white">{name}</h3>
            {Number(rating) > 0 && (
              <div className="flex items-center gap-1">
                <Star size={13} fill="#FFC107" color="#FFC107" />
                <span className="text-[13px] font-bold text-white">{rating}</span>
              </div>
            )}
          </div>

          <div className="flex items-center gap-1 mt-1.5 text-gray-500 text-[13px]">
            <MapPin size={12} />
            <span>{car.location}</span>
          </div>

          <div className="mt-3.5 flex justify-between items-center">
            <span className="text-lg font-extrabold text-accent">
              ₦{price}<span className="text-xs font-medium text-gray-500">/day</span>
            </span>
            <div className="flex gap-1.5">
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
  return <span className="bg-white/[0.06] px-2.5 py-1 rounded-lg text-[11px] font-semibold text-gray-400">{children}</span>;
}
