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
      <div className="max-w-[500px] mx-auto pt-[120px] px-6 pb-[120px] text-center">
        <p className="text-white/40">Missing booking info.</p>
        <button
          onClick={() => navigate('/trips')}
          className="bg-transparent border-0 text-accent text-sm font-semibold cursor-pointer mt-4 inline-block"
        >
          Back to Trips
        </button>
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
        className="max-w-[500px] mx-auto pt-[120px] px-6 pb-[120px] text-center"
      >
        <div className="w-[72px] h-[72px] rounded-3xl mx-auto mb-5 bg-accent/10 flex items-center justify-center">
          <Star size={32} color="#22C55E" fill="#22C55E" />
        </div>
        <h2 className="text-2xl font-extrabold mb-2">Thanks for your review!</h2>
        <p className="text-white/40 text-sm mb-8">
          Your feedback helps the Qent community.
        </p>
        <button
          onClick={() => navigate('/trips')}
          className="w-full p-4 bg-accent text-black border-0 rounded-2xl text-[15px] font-bold cursor-pointer transition-opacity"
        >
          Back to Trips
        </button>
      </motion.div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      className="max-w-[500px] mx-auto pt-[100px] px-6 pb-20"
    >
      <button
        onClick={() => navigate(-1)}
        className="flex items-center justify-center w-10 h-10 rounded-xl mb-5 bg-white/[0.06] border border-white/[0.08] text-white cursor-pointer"
      >
        <ArrowLeft size={18} />
      </button>

      <h1 className="text-[28px] font-extrabold tracking-tight mb-1.5">Leave a Review</h1>
      <p className="text-white/40 text-sm mb-9">
        How was your experience with {carName}?
      </p>

      {error && (
        <div className="bg-red-500/10 text-red-500 px-4 py-3 rounded-2xl text-[13px] font-medium mb-5 border border-red-500/20">
          {error}
        </div>
      )}

      {/* Star rating */}
      <div className="mb-8">
        <p className="text-[13px] font-semibold text-white/50 mb-3 uppercase tracking-wider">Rating</p>
        <div className="flex gap-2">
          {[1, 2, 3, 4, 5].map(n => {
            const filled = hovered >= n || rating >= n;
            return (
              <button
                key={n}
                onClick={() => setRating(n)}
                onMouseEnter={() => setHovered(n)}
                onMouseLeave={() => setHovered(0)}
                className={`bg-transparent border-0 cursor-pointer p-1 transition-transform duration-150 ${
                  filled ? 'scale-110' : 'scale-100'
                }`}
              >
                <Star
                  size={36}
                  color={filled ? '#F59E0B' : 'rgba(255,255,255,0.15)'}
                  fill={filled ? '#F59E0B' : 'transparent'}
                  style={{ transition: 'all 0.15s' }}
                />
              </button>
            );
          })}
        </div>
        {rating > 0 && (
          <p className="text-[13px] text-white/40 mt-2">
            {['', 'Poor', 'Fair', 'Good', 'Great', 'Excellent'][rating]}
          </p>
        )}
      </div>

      {/* Comment */}
      <div className="mb-8">
        <p className="text-[13px] font-semibold text-white/50 mb-3 uppercase tracking-wider">Comment (optional)</p>
        <textarea
          value={comment}
          onChange={e => setComment(e.target.value)}
          placeholder="Tell us about your experience..."
          rows={4}
          className="w-full px-4 py-3.5 rounded-2xl text-sm bg-white/[0.04] border border-white/[0.08] text-white resize-y outline-none box-border transition-colors focus:border-accent/40"
        />
      </div>

      {/* Submit */}
      <button
        onClick={handleSubmit}
        disabled={submitting || rating === 0}
        className={`w-full p-4 bg-accent text-black border-0 rounded-2xl text-[15px] font-bold transition-opacity ${
          (submitting || rating === 0) ? 'opacity-50 cursor-not-allowed' : 'opacity-100 cursor-pointer'
        }`}
      >
        {submitting ? 'Submitting...' : 'Submit Review'}
      </button>
    </motion.div>
  );
}
