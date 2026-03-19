import { useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { motion } from 'framer-motion';
import { Star, ArrowLeft } from 'lucide-react';
import { createReview } from '../utils/api';
import { useAuth } from '../hooks/useAuth';

export default function LeaveReview() {
  const { user } = useAuth();
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const bookingId = params.get('booking');
  const revieweeId = params.get('reviewee');
  const carName = params.get('car') || 'your trip';

  const [rating, setRating] = useState(0);
  const [hovered, setHovered] = useState(0);
  const [comment, setComment] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState('');
  const [done, setDone] = useState(false);

  if (!user) { navigate('/login'); return null; }
  if (!bookingId || !revieweeId) {
    return (
      <div style={{ maxWidth: 500, margin: '0 auto', padding: '120px 24px', textAlign: 'center' }}>
        <p style={{ color: 'rgba(255,255,255,0.4)' }}>Missing booking info.</p>
        <button onClick={() => navigate('/trips')} style={linkStyle}>Back to Trips</button>
      </div>
    );
  }

  const handleSubmit = async () => {
    if (rating === 0) { setError('Please select a rating'); return; }
    setSubmitting(true);
    setError('');
    try {
      await createReview({
        booking_id: bookingId,
        reviewee_id: revieweeId,
        rating,
        comment: comment.trim() || null,
      });
      setDone(true);
    } catch (e) {
      setError(e.response?.data?.error || 'Failed to submit review');
    }
    setSubmitting(false);
  };

  if (done) {
    return (
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        style={{ maxWidth: 500, margin: '0 auto', padding: '120px 24px', textAlign: 'center' }}
      >
        <div style={{
          width: 72, height: 72, borderRadius: 24, margin: '0 auto 20px',
          background: 'rgba(34,197,94,0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center',
        }}>
          <Star size={32} color="#22C55E" fill="#22C55E" />
        </div>
        <h2 style={{ fontSize: 24, fontWeight: 800, marginBottom: 8 }}>Thanks for your review!</h2>
        <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14, marginBottom: 32 }}>
          Your feedback helps the Qent community.
        </p>
        <button onClick={() => navigate('/trips')} style={primaryBtn}>Back to Trips</button>
      </motion.div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      style={{ maxWidth: 500, margin: '0 auto', padding: '100px 24px 80px' }}
    >
      <button onClick={() => navigate(-1)} style={backBtn}>
        <ArrowLeft size={18} />
      </button>

      <h1 style={{ fontSize: 28, fontWeight: 800, letterSpacing: -0.5, marginBottom: 6 }}>Leave a Review</h1>
      <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14, marginBottom: 36 }}>
        How was your experience with {carName}?
      </p>

      {error && <div style={errorBox}>{error}</div>}

      {/* Star rating */}
      <div style={{ marginBottom: 32 }}>
        <p style={{ fontSize: 13, fontWeight: 600, color: 'rgba(255,255,255,0.5)', marginBottom: 12, textTransform: 'uppercase', letterSpacing: 0.5 }}>Rating</p>
        <div style={{ display: 'flex', gap: 8 }}>
          {[1, 2, 3, 4, 5].map(n => (
            <button
              key={n}
              onClick={() => setRating(n)}
              onMouseEnter={() => setHovered(n)}
              onMouseLeave={() => setHovered(0)}
              style={{
                background: 'none', border: 'none', cursor: 'pointer', padding: 4,
                transition: 'transform 0.15s',
                transform: (hovered >= n || rating >= n) ? 'scale(1.15)' : 'scale(1)',
              }}
            >
              <Star
                size={36}
                color={(hovered >= n || rating >= n) ? '#F59E0B' : 'rgba(255,255,255,0.15)'}
                fill={(hovered >= n || rating >= n) ? '#F59E0B' : 'transparent'}
                style={{ transition: 'all 0.15s' }}
              />
            </button>
          ))}
        </div>
        {rating > 0 && (
          <p style={{ fontSize: 13, color: 'rgba(255,255,255,0.4)', marginTop: 8 }}>
            {['', 'Poor', 'Fair', 'Good', 'Great', 'Excellent'][rating]}
          </p>
        )}
      </div>

      {/* Comment */}
      <div style={{ marginBottom: 32 }}>
        <p style={{ fontSize: 13, fontWeight: 600, color: 'rgba(255,255,255,0.5)', marginBottom: 12, textTransform: 'uppercase', letterSpacing: 0.5 }}>Comment (optional)</p>
        <textarea
          value={comment}
          onChange={e => setComment(e.target.value)}
          placeholder="Tell us about your experience..."
          rows={4}
          style={{
            width: '100%', padding: '14px 16px', borderRadius: 14, fontSize: 14,
            background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)',
            color: 'white', fontFamily: 'inherit', resize: 'vertical', outline: 'none',
            boxSizing: 'border-box', transition: 'border-color 0.2s',
          }}
          onFocus={e => e.target.style.borderColor = 'rgba(34,197,94,0.4)'}
          onBlur={e => e.target.style.borderColor = 'rgba(255,255,255,0.08)'}
        />
      </div>

      {/* Submit */}
      <button
        onClick={handleSubmit}
        disabled={submitting || rating === 0}
        style={{
          ...primaryBtn,
          opacity: (submitting || rating === 0) ? 0.5 : 1,
          cursor: (submitting || rating === 0) ? 'not-allowed' : 'pointer',
        }}
      >
        {submitting ? 'Submitting...' : 'Submit Review'}
      </button>
    </motion.div>
  );
}

const backBtn = {
  display: 'flex', alignItems: 'center', justifyContent: 'center',
  width: 40, height: 40, borderRadius: 12, marginBottom: 20,
  background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
  color: 'white', cursor: 'pointer',
};

const primaryBtn = {
  width: '100%', padding: 16, background: '#22C55E', color: '#0A0A0A',
  border: 'none', borderRadius: 14, fontSize: 15, fontWeight: 700,
  cursor: 'pointer', fontFamily: 'inherit', transition: 'opacity 0.2s',
};

const linkStyle = {
  background: 'none', border: 'none', color: '#22C55E',
  fontSize: 14, fontWeight: 600, cursor: 'pointer', fontFamily: 'inherit',
  marginTop: 16, display: 'inline-block',
};

const errorBox = {
  background: 'rgba(239,68,68,0.1)', color: '#EF4444',
  padding: '12px 16px', borderRadius: 14, fontSize: 13,
  fontWeight: 500, marginBottom: 20, border: '1px solid rgba(239,68,68,0.2)',
};
