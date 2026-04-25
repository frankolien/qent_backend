import { useState, useEffect, useRef, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { motion, AnimatePresence } from 'framer-motion';
import { ArrowLeft, Send, MessageSquare } from 'lucide-react';
import { getMessages, sendMessage, getConversations } from '../utils/api';
import { useAuth } from '../hooks/useAuth';

function fmtTime(dateStr) {
  if (!dateStr) return '';
  const d = new Date(dateStr);
  return d.toLocaleTimeString('en-NG', { hour: '2-digit', minute: '2-digit', hour12: true });
}

function fmtDateHeader(dateStr) {
  if (!dateStr) return '';
  const d = new Date(dateStr);
  const today = new Date();
  const yesterday = new Date();
  yesterday.setDate(today.getDate() - 1);

  const isSameDay = (a, b) =>
    a.getFullYear() === b.getFullYear() &&
    a.getMonth() === b.getMonth() &&
    a.getDate() === b.getDate();

  if (isSameDay(d, today)) return 'Today';
  if (isSameDay(d, yesterday)) return 'Yesterday';
  return d.toLocaleDateString('en-NG', { month: 'short', day: 'numeric', year: 'numeric' });
}

function groupByDate(messages) {
  const groups = [];
  let currentDate = null;
  messages.forEach(msg => {
    const dateStr = new Date(msg.created_at).toDateString();
    if (dateStr !== currentDate) {
      currentDate = dateStr;
      groups.push({ type: 'header', date: msg.created_at, key: 'h-' + msg.created_at });
    }
    groups.push({ type: 'message', ...msg });
  });
  return groups;
}

export default function ChatDetail() {
  const { id } = useParams();
  const { user } = useAuth();
  const navigate = useNavigate();
  const [messages, setMessages] = useState([]);
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [input, setInput] = useState('');
  const [otherName, setOtherName] = useState('Chat');
  const [error, setError] = useState('');
  const bottomRef = useRef(null);
  const inputRef = useRef(null);
  const pollingRef = useRef(null);

  const fetchMessages = useCallback(async (silent = false) => {
    try {
      const r = await getMessages(id);
      setMessages(r.data || []);
      if (!silent) setLoading(false);
    } catch {
      if (!silent) {
        setError('Failed to load messages');
        setLoading(false);
      }
    }
  }, [id]);

  useEffect(() => {
    if (!user) { navigate('/login'); return; }
    getConversations()
      .then(r => {
        const convs = r.data || [];
        const conv = convs.find(c => String(c.id) === String(id));
        if (conv) {
          const name = conv.other_user_name ||
            (conv.renter_id === user.id ? conv.host_name : conv.renter_name) ||
            'Chat';
          setOtherName(name);
        }
      })
      .catch(() => {});
  }, [id, user]);

  useEffect(() => {
    fetchMessages(false);
    pollingRef.current = setInterval(() => fetchMessages(true), 5000);
    return () => clearInterval(pollingRef.current);
  }, [fetchMessages]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSend = async () => {
    const content = input.trim();
    if (!content || sending) return;

    const optimistic = {
      id: 'opt-' + Date.now(),
      sender_id: user?.id,
      content,
      created_at: new Date().toISOString(),
      optimistic: true,
    };
    setMessages(prev => [...prev, optimistic]);
    setInput('');
    setSending(true);
    inputRef.current?.focus();

    try {
      await sendMessage(id, content);
      await fetchMessages(true);
    } catch {
      setMessages(prev => prev.filter(m => m.id !== optimistic.id));
      setInput(content);
      setError('Failed to send message');
    }
    setSending(false);
  };

  const handleKeyDown = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const grouped = groupByDate(messages);
  const inputReady = input.trim() && !sending;

  return (
    <div className="flex flex-col h-screen bg-black font-sans">
      {/* Header */}
      <motion.div
        initial={{ opacity: 0, y: -10 }}
        animate={{ opacity: 1, y: 0 }}
        className="fixed top-0 left-0 right-0 z-[100] px-6 py-4 bg-black/90 backdrop-blur-nav border-b border-white/[0.07] flex items-center gap-3.5"
      >
        <button
          onClick={() => navigate('/messages')}
          className="w-[38px] h-[38px] rounded-xl flex-shrink-0 bg-white/[0.06] border border-white/[0.08] text-white cursor-pointer flex items-center justify-center"
        >
          <ArrowLeft size={18} />
        </button>
        <div
          className="w-[38px] h-[38px] rounded-xl border border-accent/20 flex items-center justify-center text-sm font-bold text-accent flex-shrink-0"
          style={{ background: 'linear-gradient(135deg, rgba(34,197,94,0.3), rgba(34,197,94,0.1))' }}
        >
          {otherName[0]?.toUpperCase() || '?'}
        </div>
        <div className="flex-1">
          <div className="text-base font-bold text-white">{otherName}</div>
          <div className="text-[11px] text-white/40">
            {loading ? 'Loading…' : `${messages.length} message${messages.length !== 1 ? 's' : ''}`}
          </div>
        </div>
      </motion.div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto pt-20 px-4 pb-[100px] flex flex-col">
        {error && (
          <div className="bg-red-500/10 text-red-500 px-3.5 py-2.5 rounded-xl text-[13px] font-medium border border-red-500/20 mb-4 self-center max-w-[400px]">
            {error}
          </div>
        )}

        {loading ? (
          <div className="flex flex-col items-center justify-center flex-1 gap-4">
            <div className="spinner-sm" />
            <p className="text-white/40 text-sm">Loading messages…</p>
          </div>
        ) : messages.length === 0 ? (
          <div className="flex flex-col items-center justify-center flex-1 gap-4">
            <div className="w-16 h-16 rounded-[20px] bg-accent/[0.08] border border-accent/15 flex items-center justify-center">
              <MessageSquare size={28} color="rgba(34,197,94,0.5)" />
            </div>
            <p className="text-white/40 text-sm text-center">No messages yet. Say hello!</p>
          </div>
        ) : (
          <div className="flex flex-col gap-1 max-w-[700px] w-full self-center">
            <AnimatePresence initial={false}>
              {grouped.map(item => {
                if (item.type === 'header') {
                  return (
                    <div key={item.key} className="text-center mt-4 mb-2">
                      <span className="px-3 py-1 rounded-full bg-white/[0.06] text-[11px] text-white/40 font-semibold">
                        {fmtDateHeader(item.date)}
                      </span>
                    </div>
                  );
                }

                const isMine = item.sender_id === user?.id;
                return (
                  <motion.div
                    key={item.id}
                    initial={{ opacity: 0, y: 10, scale: 0.98 }}
                    animate={{ opacity: item.optimistic ? 0.7 : 1, y: 0, scale: 1 }}
                    exit={{ opacity: 0, scale: 0.95 }}
                    transition={{ duration: 0.2 }}
                    className={`flex mb-0.5 ${isMine ? 'justify-end' : 'justify-start'}`}
                  >
                    <div className="max-w-[72%]">
                      <div
                        className={`px-3.5 py-2.5 text-sm leading-relaxed break-words ${
                          isMine
                            ? 'text-black rounded-[18px_18px_4px_18px]'
                            : 'bg-white/[0.07] text-white border border-white/[0.08] rounded-[18px_18px_18px_4px]'
                        }`}
                        style={isMine ? { background: 'linear-gradient(135deg, #22C55E, #16A34A)' } : undefined}
                      >
                        {item.content}
                      </div>
                      <div
                        className={`text-[10px] text-white/30 mt-0.5 ${
                          isMine ? 'text-right pr-1' : 'text-left pl-1'
                        }`}
                      >
                        {fmtTime(item.created_at)}
                      </div>
                    </div>
                  </motion.div>
                );
              })}
            </AnimatePresence>
            <div ref={bottomRef} />
          </div>
        )}
      </div>

      {/* Input bar */}
      <div className="fixed bottom-0 left-0 right-0 px-4 pt-3 pb-5 bg-black/95 backdrop-blur-nav border-t border-white/[0.07]">
        <div className="max-w-[700px] mx-auto flex gap-2.5 items-end">
          <textarea
            ref={inputRef}
            value={input}
            onChange={e => {
              setInput(e.target.value);
              e.target.style.height = 'auto';
              e.target.style.height = Math.min(e.target.scrollHeight, 120) + 'px';
            }}
            onKeyDown={handleKeyDown}
            placeholder="Type a message…"
            rows={1}
            className="flex-1 px-4 py-3 rounded-[18px] bg-white/[0.07] border border-white/10 text-white text-sm outline-none resize-none leading-relaxed max-h-[120px] overflow-y-auto transition-colors focus:border-accent/40"
          />
          <button
            onClick={handleSend}
            disabled={!inputReady}
            className={`w-[46px] h-[46px] rounded-2xl flex-shrink-0 border-0 flex items-center justify-center transition-all duration-200 ${
              inputReady ? 'bg-accent cursor-pointer' : 'bg-white/[0.08] cursor-not-allowed'
            }`}
          >
            <Send size={18} color={inputReady ? '#0A0A0A' : 'rgba(255,255,255,0.3)'} />
          </button>
        </div>
      </div>
    </div>
  );
}
