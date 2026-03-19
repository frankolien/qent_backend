import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { motion } from 'framer-motion';
import { MessageSquare, ArrowLeft, Search, ChevronRight } from 'lucide-react';
import { getConversations } from '../utils/api';
import { useAuth } from '../hooks/useAuth';

function timeAgo(dateStr) {
  if (!dateStr) return '';
  const now = new Date();
  const then = new Date(dateStr);
  const diffMs = now - then;
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHrs = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHrs / 24);

  if (diffSecs < 60) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHrs < 24) return `${diffHrs}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return then.toLocaleDateString('en-NG', { month: 'short', day: 'numeric' });
}

export default function Messages() {
  const { user } = useAuth();
  const navigate = useNavigate();
  const [conversations, setConversations] = useState([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    if (!user) { navigate('/login'); return; }
    getConversations()
      .then(r => setConversations(r.data || []))
      .catch(() => setError('Failed to load conversations'))
      .finally(() => setLoading(false));
  }, [user]);

  const filtered = conversations.filter(c => {
    const otherName = getOtherName(c, user);
    const lastMsg = c.last_message?.content || '';
    return (
      otherName.toLowerCase().includes(search.toLowerCase()) ||
      lastMsg.toLowerCase().includes(search.toLowerCase())
    );
  });

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      style={{ maxWidth: 680, margin: '0 auto', padding: '100px 24px 80px' }}
    >
      {/* Header */}
      <button onClick={() => navigate(-1)} style={backBtn}>
        <ArrowLeft size={18} />
      </button>

      <h1 style={{ fontSize: 32, fontWeight: 900, letterSpacing: -1, marginBottom: 6 }}>Messages</h1>
      <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14, marginBottom: 28 }}>
        {conversations.length} conversation{conversations.length !== 1 ? 's' : ''}
      </p>

      {/* Search */}
      {conversations.length > 0 && (
        <div style={{
          display: 'flex', alignItems: 'center', gap: 10,
          padding: '12px 16px', borderRadius: 14,
          background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)',
          marginBottom: 24,
        }}>
          <Search size={16} color="rgba(255,255,255,0.3)" />
          <input
            value={search}
            onChange={e => setSearch(e.target.value)}
            placeholder="Search conversations…"
            style={{
              flex: 1, border: 'none', background: 'none', fontSize: 14,
              color: 'white', outline: 'none', fontFamily: 'inherit',
            }}
          />
        </div>
      )}

      {error && <div style={errorBox}>{error}</div>}

      {loading ? (
        <div style={centerStyle}>
          <div style={spinnerStyle} />
          <p style={{ color: 'rgba(255,255,255,0.4)', marginTop: 16 }}>Loading conversations…</p>
        </div>
      ) : filtered.length === 0 ? (
        <EmptyState hasConvos={conversations.length > 0} navigate={navigate} />
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          {filtered.map((conv, i) => (
            <ConvRow
              key={conv.id}
              conv={conv}
              user={user}
              index={i}
              onClick={() => navigate(`/messages/${conv.id}`)}
            />
          ))}
        </div>
      )}
    </motion.div>
  );
}

// ─── Conversation Row ─────────────────────────────────────────────────────────

function ConvRow({ conv, user, index, onClick }) {
  const [hovered, setHovered] = useState(false);
  const otherName = getOtherName(conv, user);
  const initials = otherName.split(' ').map(n => n[0]).join('').toUpperCase().slice(0, 2);
  const lastMsg = conv.last_message?.content || 'No messages yet';
  const unread = conv.unread_count || 0;
  const time = timeAgo(conv.last_message?.created_at || conv.updated_at);
  const isLastFromMe = conv.last_message?.sender_id === user?.id;

  return (
    <motion.div
      initial={{ opacity: 0, x: -12 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.3, delay: index * 0.05 }}
      onClick={onClick}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: 'flex', alignItems: 'center', gap: 14, padding: '14px 16px',
        borderRadius: 16, cursor: 'pointer', transition: 'background 0.2s',
        background: hovered ? 'rgba(255,255,255,0.06)' : 'transparent',
      }}
    >
      {/* Avatar */}
      <div style={{ position: 'relative', flexShrink: 0 }}>
        <div style={{
          width: 50, height: 50, borderRadius: 16,
          background: 'linear-gradient(135deg, rgba(34,197,94,0.3), rgba(34,197,94,0.1))',
          border: '1px solid rgba(34,197,94,0.2)',
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          fontSize: 16, fontWeight: 700, color: 'var(--accent)',
        }}>
          {initials || '?'}
        </div>
        {unread > 0 && (
          <div style={{
            position: 'absolute', top: -4, right: -4,
            minWidth: 18, height: 18, borderRadius: 9,
            background: '#22C55E', color: '#0A0A0A',
            fontSize: 10, fontWeight: 800,
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            padding: '0 4px', border: '2px solid #0A0A0A',
          }}>
            {unread > 99 ? '99+' : unread}
          </div>
        )}
      </div>

      {/* Text */}
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline', marginBottom: 4 }}>
          <span style={{ fontSize: 15, fontWeight: unread > 0 ? 800 : 600, color: 'white' }}>{otherName}</span>
          <span style={{ fontSize: 11, color: 'rgba(255,255,255,0.35)', flexShrink: 0, marginLeft: 8 }}>{time}</span>
        </div>
        <p style={{
          margin: 0, fontSize: 13,
          color: unread > 0 ? 'rgba(255,255,255,0.7)' : 'rgba(255,255,255,0.4)',
          fontWeight: unread > 0 ? 500 : 400,
          overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        }}>
          {isLastFromMe && <span style={{ color: 'rgba(34,197,94,0.7)' }}>You: </span>}
          {lastMsg}
        </p>
      </div>

      <ChevronRight size={16} color="rgba(255,255,255,0.2)" style={{ flexShrink: 0 }} />
    </motion.div>
  );
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

function getOtherName(conv, user) {
  if (!user) return conv.other_user_name || 'User';
  if (conv.other_user_name) return conv.other_user_name;
  if (conv.renter_id === user.id) return conv.host_name || 'Host';
  return conv.renter_name || 'Renter';
}

function EmptyState({ hasConvos, navigate }) {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.97 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.4 }}
      style={{ textAlign: 'center', padding: '80px 24px' }}
    >
      <div style={{
        width: 80, height: 80, borderRadius: 24,
        background: 'rgba(34,197,94,0.08)', border: '1px solid rgba(34,197,94,0.15)',
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        margin: '0 auto 24px',
      }}>
        <MessageSquare size={36} color="rgba(34,197,94,0.5)" />
      </div>
      <h3 style={{ fontSize: 22, fontWeight: 800, marginBottom: 10 }}>
        {hasConvos ? 'No results found' : 'No conversations yet'}
      </h3>
      <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14, lineHeight: 1.6, maxWidth: 320, margin: '0 auto 32px' }}>
        {hasConvos
          ? 'No conversations match your search.'
          : 'When you book a car or receive a booking, your messages will appear here.'}
      </p>
      {!hasConvos && (
        <button
          onClick={() => navigate('/search')}
          style={{
            padding: '14px 32px', background: 'var(--accent)', color: '#0A0A0A',
            border: 'none', borderRadius: 100, fontSize: 15, fontWeight: 700,
            cursor: 'pointer', fontFamily: 'inherit',
          }}
        >
          Browse cars
        </button>
      )}
    </motion.div>
  );
}

// ─── Shared styles ───────────────────────────────────────────────────────────

const backBtn = {
  display: 'flex', alignItems: 'center', justifyContent: 'center',
  width: 40, height: 40, borderRadius: 12, marginBottom: 20,
  background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
  color: 'white', cursor: 'pointer',
};

const errorBox = {
  background: 'rgba(239,68,68,0.1)', color: '#EF4444',
  padding: '12px 16px', borderRadius: 14, fontSize: 13,
  fontWeight: 500, marginBottom: 20, border: '1px solid rgba(239,68,68,0.2)',
};

const centerStyle = {
  display: 'flex', flexDirection: 'column', alignItems: 'center',
  justifyContent: 'center', padding: '80px 24px',
};

const spinnerStyle = {
  width: 36, height: 36, borderRadius: '50%',
  border: '3px solid rgba(255,255,255,0.08)',
  borderTopColor: '#22C55E',
  animation: 'spin 0.8s linear infinite',
};
