import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { motion } from 'framer-motion';
import { Calendar, MapPin, Shield, ChevronLeft } from 'lucide-react';
import { getCar, createBooking } from '../utils/api';
import { useAuth } from '../hooks/useAuth';

export default function BookCar() {
  const { id } = useParams();
  const navigate = useNavigate();
  const { user } = useAuth();
  const [car, setCar] = useState(null);
  const [loading, setLoading] = useState(true);
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    if (!user) { navigate('/login'); return; }
    getCar(id).then(r => setCar(r.data)).catch(() => navigate('/')).finally(() => setLoading(false));
  }, [id, user]);

  if (loading || !car) {
    return (
      <div className="min-h-screen flex items-center justify-center text-gray-500">Loading...</div>
    );
  }

  const photo = car.photos?.[0] || '';
  const name = `${car.make} ${car.model} ${car.year}`;
  const pricePerDay = car.price_per_day;

  let totalDays = 0;
  let subtotal = 0;
  let serviceFee = 0;
  let total = 0;
  if (startDate && endDate) {
    const s = new Date(startDate);
    const e = new Date(endDate);
    totalDays = Math.max(1, Math.ceil((e - s) / (1000 * 60 * 60 * 24)));
    subtotal = pricePerDay * totalDays;
    serviceFee = subtotal * 0.1;
    total = subtotal + serviceFee;
  }

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');

    if (!startDate || !endDate) { setError('Please select dates'); return; }
    if (new Date(endDate) <= new Date(startDate)) { setError('Return date must be after pickup date'); return; }

    setSubmitting(true);
    try {
      const res = await createBooking({ car_id: car.id, start_date: startDate, end_date: endDate });
      navigate('/booking-success', { state: { booking: res.data, car } });
    } catch (err) {
      setError(err.response?.data?.error || 'Booking failed');
    }
    setSubmitting(false);
  };

  const fmt = (n) => Number(n).toLocaleString();

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="max-w-[900px] mx-auto pt-[100px] px-8 pb-20"
    >
      <button
        onClick={() => navigate(-1)}
        className="bg-white/[0.06] border border-white/[0.08] rounded-2xl w-11 h-11 cursor-pointer flex items-center justify-center text-white mb-7"
      >
        <ChevronLeft size={20} />
      </button>

      <h1 className="text-[28px] font-extrabold mb-8">Complete your booking</h1>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        {/* Left — Form */}
        <div>
          <form onSubmit={handleSubmit}>
            {error && (
              <div className="bg-red-500/10 text-red-500 px-4 py-3 rounded-2xl text-[13px] font-medium mb-5 border border-red-500/20">
                {error}
              </div>
            )}

            <Label>Pickup Date</Label>
            <InputWrap>
              <Calendar size={16} color="var(--gray-500)" />
              <input
                type="date"
                value={startDate}
                onChange={e => setStartDate(e.target.value)}
                min={new Date().toISOString().split('T')[0]}
                className="flex-1 border-0 bg-transparent text-white text-sm outline-none"
                style={{ colorScheme: 'dark' }}
              />
            </InputWrap>

            <Label>Return Date</Label>
            <InputWrap>
              <Calendar size={16} color="var(--gray-500)" />
              <input
                type="date"
                value={endDate}
                onChange={e => setEndDate(e.target.value)}
                min={startDate || new Date().toISOString().split('T')[0]}
                className="flex-1 border-0 bg-transparent text-white text-sm outline-none"
                style={{ colorScheme: 'dark' }}
              />
            </InputWrap>

            <Label>Pickup Location</Label>
            <InputWrap>
              <MapPin size={16} color="var(--gray-500)" />
              <span className="text-white text-sm">{car.location}</span>
            </InputWrap>

            {totalDays > 0 && (
              <div className="mt-7 p-6 rounded-[20px] bg-white/[0.03] border border-white/[0.06]">
                <Row label={`₦${fmt(pricePerDay)} x ${totalDays} day${totalDays > 1 ? 's' : ''}`} value={`₦${fmt(subtotal)}`} />
                <Row label="Service fee (10%)" value={`₦${fmt(serviceFee)}`} />
                <div className="h-px bg-white/[0.08] my-3.5" />
                <Row label="Total" value={`₦${fmt(total)}`} bold />
              </div>
            )}

            <button
              type="submit"
              disabled={submitting || !totalDays}
              className={`w-full p-4 mt-6 border-0 rounded-2xl text-base font-bold transition-opacity ${
                totalDays
                  ? 'bg-accent text-black cursor-pointer'
                  : 'bg-white/10 text-gray-500 cursor-not-allowed'
              } ${submitting ? 'opacity-60' : 'opacity-100'}`}
            >
              {submitting ? 'Booking...' : totalDays ? `Book for ₦${fmt(total)}` : 'Select dates to continue'}
            </button>
          </form>
        </div>

        {/* Right — Car summary */}
        <div>
          <div className="rounded-3xl overflow-hidden bg-white/[0.04] border border-white/[0.08] sticky top-[100px]">
            {photo && <img src={photo} alt={name} className="w-full h-[220px] object-cover" />}
            <div className="p-6">
              <h3 className="text-xl font-bold mb-2">{name}</h3>
              <div className="flex items-center gap-1.5 text-gray-500 text-[13px] mb-4">
                <MapPin size={13} /> {car.location}
              </div>
              <div className="text-2xl font-extrabold text-accent">
                ₦{fmt(pricePerDay)}<span className="text-[13px] font-medium text-gray-500">/day</span>
              </div>

              <div className="mt-5 p-4 rounded-2xl bg-accent/[0.08] border border-accent/15 flex items-center gap-2.5">
                <Shield size={16} color="var(--accent)" />
                <span className="text-[13px] text-accent font-semibold">Protection plan included</span>
              </div>

              <div className="mt-4 flex gap-2 flex-wrap">
                <Tag>{car.seats || 5} seats</Tag>
                <Tag>{car.color}</Tag>
                {car.features?.slice(0, 3).map((f, i) => <Tag key={i}>{f}</Tag>)}
              </div>
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );
}

function Label({ children }) {
  return <label className="text-[13px] font-semibold text-gray-400 block mb-2 mt-5">{children}</label>;
}

function InputWrap({ children }) {
  return (
    <div className="flex items-center gap-3 px-4 py-3.5 bg-white/[0.04] border border-white/[0.08] rounded-2xl">
      {children}
    </div>
  );
}

function Row({ label, value, bold }) {
  return (
    <div className="flex justify-between mb-2.5">
      <span className={`text-sm ${bold ? 'text-white font-bold' : 'text-gray-500 font-normal'}`}>{label}</span>
      <span className={`text-sm text-white ${bold ? 'font-bold' : 'font-medium'}`}>{value}</span>
    </div>
  );
}

function Tag({ children }) {
  return <span className="bg-white/[0.06] px-3 py-[5px] rounded-lg text-xs font-medium text-gray-400">{children}</span>;
}
