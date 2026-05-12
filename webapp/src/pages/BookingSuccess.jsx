import { useLocation, useNavigate, Link } from 'react-router-dom';
import { motion } from 'framer-motion';
import { CheckCircle, ArrowRight } from 'lucide-react';

export default function BookingSuccess() {
  const { state } = useLocation();
  const navigate = useNavigate();

  if (!state?.booking) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <p className="text-gray-500 mb-4">No booking found</p>
          <Link to="/" className="text-accent font-semibold">Go home</Link>
        </div>
      </div>
    );
  }

  const { booking, car } = state;

  return (
    <div className="min-h-screen flex items-center justify-center p-6">
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.5 }}
        className="text-center max-w-[480px]"
      >
        <motion.div
          initial={{ scale: 0 }}
          animate={{ scale: 1 }}
          transition={{ type: 'spring', delay: 0.2, stiffness: 200 }}
        >
          <div className="w-20 h-20 rounded-full bg-accent/10 border-2 border-accent/30 flex items-center justify-center mx-auto mb-7">
            <CheckCircle size={36} color="var(--accent)" />
          </div>
        </motion.div>

        <h1 className="text-[32px] font-black mb-3">Booking Submitted!</h1>
        <p className="text-gray-500 text-[15px] leading-relaxed mb-8">
          Your booking for <strong className="text-white">{car?.make} {car?.model}</strong> has been sent to the host.
          You'll be notified once they accept.
        </p>

        <div className="p-6 rounded-[20px] bg-white/[0.04] border border-white/[0.08] text-left mb-8">
          <Row label="Booking ID" value={`#${booking.id?.substring(0, 8)}`} />
          <Row label="Status" value="Pending Approval" accent />
          <Row label="Dates" value={`${booking.start_date} → ${booking.end_date}`} />
          <Row label="Total" value={`₦${Number(booking.total_amount).toLocaleString()}`} />
        </div>

        <div className="flex gap-3">
          <button
            onClick={() => navigate('/')}
            className="flex-1 p-4 bg-white/[0.06] text-white border border-white/[0.08] rounded-2xl text-sm font-semibold cursor-pointer"
          >
            Browse more
          </button>
          <button
            onClick={() => navigate('/trips')}
            className="flex-1 p-4 bg-accent text-black rounded-2xl text-sm font-semibold cursor-pointer flex items-center justify-center gap-1.5"
          >
            My trips <ArrowRight size={16} />
          </button>
        </div>
      </motion.div>
    </div>
  );
}

function Row({ label, value, accent }) {
  return (
    <div className="flex justify-between mb-3">
      <span className="text-gray-500 text-[13px]">{label}</span>
      <span className={`text-[13px] font-semibold ${accent ? 'text-accent' : 'text-white'}`}>{value}</span>
    </div>
  );
}
