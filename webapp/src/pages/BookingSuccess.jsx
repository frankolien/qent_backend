import { useLocation, useNavigate, Link } from 'react-router-dom';
import { motion } from 'framer-motion';
import { CheckCircle, ArrowRight } from 'lucide-react';

export default function BookingSuccess() {
  const { state } = useLocation();
  const navigate = useNavigate();

  if (!state?.booking) {
    return (
      <div style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <div style={{ textAlign: 'center' }}>
          <p style={{ color: 'var(--gray-500)', marginBottom: 16 }}>No booking found</p>
          <Link to="/" style={{ color: 'var(--accent)', fontWeight: 600 }}>Go home</Link>
        </div>
      </div>
    );
  }

  const { booking, car } = state;

  return (
    <div style={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center', padding: 24 }}>
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.5 }}
        style={{ textAlign: 'center', maxWidth: 480 }}
      >
        <motion.div
          initial={{ scale: 0 }}
          animate={{ scale: 1 }}
          transition={{ type: 'spring', delay: 0.2, stiffness: 200 }}
        >
          <div style={{
            width: 80, height: 80, borderRadius: '50%',
            background: 'rgba(34,197,94,0.1)', border: '2px solid rgba(34,197,94,0.3)',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            margin: '0 auto 28px',
          }}>
            <CheckCircle size={36} color="var(--accent)" />
          </div>
        </motion.div>

        <h1 style={{ fontSize: 32, fontWeight: 900, marginBottom: 12 }}>Booking Submitted!</h1>
        <p style={{ color: 'var(--gray-500)', fontSize: 15, lineHeight: 1.7, marginBottom: 32 }}>
          Your booking for <strong style={{ color: 'white' }}>{car?.make} {car?.model}</strong> has been sent to the host.
          You'll be notified once they accept.
        </p>

        <div style={{
          padding: 24, borderRadius: 20,
          background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)',
          textAlign: 'left', marginBottom: 32,
        }}>
          <Row label="Booking ID" value={`#${booking.id?.substring(0, 8)}`} />
          <Row label="Status" value="Pending Approval" accent />
          <Row label="Dates" value={`${booking.start_date} → ${booking.end_date}`} />
          <Row label="Total" value={`\u20A6${Number(booking.total_amount).toLocaleString()}`} />
        </div>

        <div style={{ display: 'flex', gap: 12 }}>
          <button onClick={() => navigate('/')} style={{
            flex: 1, padding: 16, background: 'rgba(255,255,255,0.06)', color: 'white',
            border: '1px solid rgba(255,255,255,0.08)', borderRadius: 14, fontSize: 14, fontWeight: 600, cursor: 'pointer', fontFamily: 'inherit',
          }}>Browse more</button>
          <button onClick={() => navigate('/trips')} style={{
            flex: 1, padding: 16, background: 'var(--accent)', color: 'var(--black)',
            border: 'none', borderRadius: 14, fontSize: 14, fontWeight: 600, cursor: 'pointer', fontFamily: 'inherit',
            display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 6,
          }}>My trips <ArrowRight size={16} /></button>
        </div>
      </motion.div>
    </div>
  );
}

function Row({ label, value, accent }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 12 }}>
      <span style={{ color: 'var(--gray-500)', fontSize: 13 }}>{label}</span>
      <span style={{ color: accent ? 'var(--accent)' : 'white', fontSize: 13, fontWeight: 600 }}>{value}</span>
    </div>
  );
}
