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

  if (loading || !car) return <div style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--gray-500)' }}>Loading...</div>;

  const photo = car.photos?.[0] || '';
  const name = `${car.make} ${car.model} ${car.year}`;
  const pricePerDay = car.price_per_day;

  // Calculate totals
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
      style={{ maxWidth: 900, margin: '0 auto', padding: '100px 32px 80px' }}
    >
      <button onClick={() => navigate(-1)} style={{
        background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
        borderRadius: 14, width: 44, height: 44, cursor: 'pointer',
        display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'white', marginBottom: 28,
      }}>
        <ChevronLeft size={20} />
      </button>

      <h1 style={{ fontSize: 28, fontWeight: 800, marginBottom: 32 }}>Complete your booking</h1>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 32 }}>
        {/* Left — Form */}
        <div>
          <form onSubmit={handleSubmit}>
            {error && (
              <div style={{ background: 'rgba(239,68,68,0.1)', color: '#EF4444', padding: '12px 16px', borderRadius: 14, fontSize: 13, fontWeight: 500, marginBottom: 20, border: '1px solid rgba(239,68,68,0.2)' }}>{error}</div>
            )}

            <label style={labelStyle}>Pickup Date</label>
            <div style={inputWrapStyle}>
              <Calendar size={16} color="var(--gray-500)" />
              <input type="date" value={startDate} onChange={e => setStartDate(e.target.value)}
                min={new Date().toISOString().split('T')[0]}
                style={dateInputStyle} />
            </div>

            <label style={labelStyle}>Return Date</label>
            <div style={inputWrapStyle}>
              <Calendar size={16} color="var(--gray-500)" />
              <input type="date" value={endDate} onChange={e => setEndDate(e.target.value)}
                min={startDate || new Date().toISOString().split('T')[0]}
                style={dateInputStyle} />
            </div>

            <label style={labelStyle}>Pickup Location</label>
            <div style={inputWrapStyle}>
              <MapPin size={16} color="var(--gray-500)" />
              <span style={{ color: 'white', fontSize: 14 }}>{car.location}</span>
            </div>

            {/* Price breakdown */}
            {totalDays > 0 && (
              <div style={{
                marginTop: 28, padding: 24, borderRadius: 20,
                background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.06)',
              }}>
                <Row label={`\u20A6${fmt(pricePerDay)} x ${totalDays} day${totalDays > 1 ? 's' : ''}`} value={`\u20A6${fmt(subtotal)}`} />
                <Row label="Service fee (10%)" value={`\u20A6${fmt(serviceFee)}`} />
                <div style={{ height: 1, background: 'rgba(255,255,255,0.08)', margin: '14px 0' }} />
                <Row label="Total" value={`\u20A6${fmt(total)}`} bold />
              </div>
            )}

            <button type="submit" disabled={submitting || !totalDays} style={{
              width: '100%', padding: 16, marginTop: 24,
              background: totalDays ? 'var(--accent)' : 'rgba(255,255,255,0.1)',
              color: totalDays ? 'var(--black)' : 'var(--gray-500)',
              border: 'none', borderRadius: 16, fontSize: 16, fontWeight: 700,
              cursor: totalDays ? 'pointer' : 'not-allowed', fontFamily: 'inherit',
              opacity: submitting ? 0.6 : 1,
            }}>
              {submitting ? 'Booking...' : totalDays ? `Book for \u20A6${fmt(total)}` : 'Select dates to continue'}
            </button>
          </form>
        </div>

        {/* Right — Car summary */}
        <div>
          <div style={{
            borderRadius: 24, overflow: 'hidden',
            background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)',
            position: 'sticky', top: 100,
          }}>
            {photo && <img src={photo} alt={name} style={{ width: '100%', height: 220, objectFit: 'cover' }} />}
            <div style={{ padding: 24 }}>
              <h3 style={{ fontSize: 20, fontWeight: 700, marginBottom: 8 }}>{name}</h3>
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, color: 'var(--gray-500)', fontSize: 13, marginBottom: 16 }}>
                <MapPin size={13} /> {car.location}
              </div>
              <div style={{ fontSize: 24, fontWeight: 800, color: 'var(--accent)' }}>
                {"\u20A6"}{fmt(pricePerDay)}<span style={{ fontSize: 13, fontWeight: 500, color: 'var(--gray-500)' }}>/day</span>
              </div>

              <div style={{ marginTop: 20, padding: 16, borderRadius: 14, background: 'rgba(34,197,94,0.08)', border: '1px solid rgba(34,197,94,0.15)', display: 'flex', alignItems: 'center', gap: 10 }}>
                <Shield size={16} color="var(--accent)" />
                <span style={{ fontSize: 13, color: 'var(--accent)', fontWeight: 600 }}>Protection plan included</span>
              </div>

              <div style={{ marginTop: 16, display: 'flex', gap: 8, flexWrap: 'wrap' }}>
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

function Row({ label, value, bold }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 10 }}>
      <span style={{ color: bold ? 'white' : 'var(--gray-500)', fontSize: 14, fontWeight: bold ? 700 : 400 }}>{label}</span>
      <span style={{ color: 'white', fontSize: 14, fontWeight: bold ? 700 : 500 }}>{value}</span>
    </div>
  );
}

function Tag({ children }) {
  return <span style={{ background: 'rgba(255,255,255,0.06)', padding: '5px 12px', borderRadius: 8, fontSize: 12, fontWeight: 500, color: 'var(--gray-400)' }}>{children}</span>;
}

const labelStyle = { fontSize: 13, fontWeight: 600, color: 'var(--gray-400)', display: 'block', marginBottom: 8, marginTop: 20 };

const inputWrapStyle = {
  display: 'flex', alignItems: 'center', gap: 12, padding: '14px 16px',
  background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)',
  borderRadius: 14,
};

const dateInputStyle = {
  flex: 1, border: 'none', background: 'none', color: 'white',
  fontSize: 14, outline: 'none', fontFamily: 'inherit',
  colorScheme: 'dark',
};
