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
      className="max-w-[680px] mx-auto pt-[100px] px-6 pb-20"
    >
      <button
        onClick={() => navigate(-1)}
        className="flex items-center justify-center w-10 h-10 rounded-xl mb-5 bg-white/[0.06] border border-white/[0.08] text-white cursor-pointer"
      >
        <ArrowLeft size={18} />
      </button>

      <h1 className="text-[32px] font-black tracking-tighter mb-1.5">Messages</h1>
      <p className="text-white/40 text-sm mb-7">
        {conversations.length} conversation{conversations.length !== 1 ? 's' : ''}
      </p>

      {/* Search */}
      {conversations.length > 0 && (
        <div className="flex items-center gap-2.5 px-4 py-3 rounded-2xl bg-white/[0.04] border border-white/[0.08] mb-6">
          <Search size={16} color="rgba(255,255,255,0.3)" />
          <input
            value={search}
            onChange={e => setSearch(e.target.value)}
            placeholder="Search conversations…"
            className="flex-1 border-0 bg-transparent text-sm text-white outline-none"
          />
        </div>
      )}

      {error && (
        <div className="bg-red-500/10 text-red-500 px-4 py-3 rounded-2xl text-[13px] font-medium mb-5 border border-red-500/20">
          {error}
        </div>
      )}

      {loading ? (
        <div className="flex flex-col items-center justify-center px-6 py-20">
          <div className="spinner" />
          <p className="text-white/40 mt-4">Loading conversations…</p>
        </div>
      ) : filtered.length === 0 ? (
        <EmptyState hasConvos={conversations.length > 0} navigate={navigate} />
      ) : (
        <div className="flex flex-col gap-0.5">
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
      className={`flex items-center gap-3.5 px-4 py-3.5 rounded-2xl cursor-pointer transition-colors duration-200 ${
        hovered ? 'bg-white/[0.06]' : 'bg-transparent'
      }`}
    >
      {/* Avatar */}
      <div className="relative flex-shrink-0">
        <div
          className="w-[50px] h-[50px] rounded-2xl border border-accent/20 flex items-center justify-center text-base font-bold text-accent"
          style={{ background: 'linear-gradient(135deg, rgba(34,197,94,0.3), rgba(34,197,94,0.1))' }}
        >
          {initials || '?'}
        </div>
        {unread > 0 && (
          <div className="absolute -top-1 -right-1 min-w-[18px] h-[18px] rounded-full bg-accent text-black text-[10px] font-extrabold flex items-center justify-center px-1 border-2 border-black">
            {unread > 99 ? '99+' : unread}
          </div>
        )}
      </div>

      {/* Text */}
      <div className="flex-1 min-w-0">
        <div className="flex justify-between items-baseline mb-1">
          <span className={`text-[15px] text-white ${unread > 0 ? 'font-extrabold' : 'font-semibold'}`}>{otherName}</span>
          <span className="text-[11px] text-white/35 flex-shrink-0 ml-2">{time}</span>
        </div>
        <p
          className={`m-0 text-[13px] overflow-hidden text-ellipsis whitespace-nowrap ${
            unread > 0 ? 'text-white/70 font-medium' : 'text-white/40 font-normal'
          }`}
        >
          {isLastFromMe && <span className="text-accent/70">You: </span>}
          {lastMsg}
        </p>
      </div>

      <ChevronRight size={16} color="rgba(255,255,255,0.2)" className="flex-shrink-0" />
    </motion.div>
  );
}

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
      className="text-center px-6 py-20"
    >
      <div className="w-20 h-20 rounded-3xl bg-accent/[0.08] border border-accent/15 flex items-center justify-center mx-auto mb-6">
        <MessageSquare size={36} color="rgba(34,197,94,0.5)" />
      </div>
      <h3 className="text-[22px] font-extrabold mb-2.5">
        {hasConvos ? 'No results found' : 'No conversations yet'}
      </h3>
      <p className="text-white/40 text-sm leading-relaxed max-w-[320px] mx-auto mb-8">
        {hasConvos
          ? 'No conversations match your search.'
          : 'When you book a car or receive a booking, your messages will appear here.'}
      </p>
      {!hasConvos && (
        <button
          onClick={() => navigate('/search')}
          className="px-8 py-3.5 bg-accent text-black border-0 rounded-full text-[15px] font-bold cursor-pointer"
        >
          Browse cars
        </button>
      )}
    </motion.div>
  );
}
