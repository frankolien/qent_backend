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

  // Load conversation name
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

  // Initial load + polling
  useEffect(() => {
    fetchMessages(false);

    pollingRef.current = setInterval(() => fetchMessages(true), 5000);
    return () => clearInterval(pollingRef.current);
  }, [fetchMessages]);

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSend = async () => {
    const content = input.trim();
    if (!content || sending) return;

    // Optimistic update
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
      // Remove optimistic message on failure
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

  return (
    <div style={{
      display: 'flex', flexDirection: 'column', height: '100vh',
      background: '#0A0A0A', fontFamily: 'Inter, sans-serif',
    }}>
      {/* Header */}
      <motion.div
        initial={{ opacity: 0, y: -10 }}
        animate={{ opacity: 1, y: 0 }}
        style={{
          position: 'fixed', top: 0, left: 0, right: 0, zIndex: 100,
          padding: '16px 24px',
          background: 'rgba(10,10,10,0.9)',
          backdropFilter: 'blur(24px)',
          WebkitBackdropFilter: 'blur(24px)',
          borderBottom: '1px solid rgba(255,255,255,0.07)',
          display: 'flex', alignItems: 'center', gap: 14,
        }}
      >
        <button onClick={() => navigate('/messages')} style={iconBtn}>
          <ArrowLeft size={18} />
        </button>
        <div style={{
          width: 38, height: 38, borderRadius: 12,
          background: 'linear-gradient(135deg, rgba(34,197,94,0.3), rgba(34,197,94,0.1))',
          border: '1px solid rgba(34,197,94,0.2)',
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          fontSize: 14, fontWeight: 700, color: 'var(--accent)', flexShrink: 0,
        }}>
          {otherName[0]?.toUpperCase() || '?'}
        </div>
        <div style={{ flex: 1 }}>
          <div style={{ fontSize: 16, fontWeight: 700, color: 'white' }}>{otherName}</div>
          <div style={{ fontSize: 11, color: 'rgba(255,255,255,0.4)' }}>
            {loading ? 'Loading…' : `${messages.length} message${messages.length !== 1 ? 's' : ''}`}
          </div>
        </div>
      </motion.div>

      {/* Messages */}
      <div style={{
        flex: 1, overflowY: 'auto', padding: '80px 16px 100px',
        display: 'flex', flexDirection: 'column',
      }}>
        {error && (
          <div style={{ ...errorBox, margin: '0 0 16px', alignSelf: 'center', maxWidth: 400 }}>{error}</div>
        )}

        {loading ? (
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', flex: 1, gap: 16 }}>
            <div style={spinnerStyle} />
            <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14 }}>Loading messages…</p>
          </div>
        ) : messages.length === 0 ? (
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', flex: 1, gap: 16 }}>
            <div style={{
              width: 64, height: 64, borderRadius: 20,
              background: 'rgba(34,197,94,0.08)', border: '1px solid rgba(34,197,94,0.15)',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
            }}>
              <MessageSquare size={28} color="rgba(34,197,94,0.5)" />
            </div>
            <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14, textAlign: 'center' }}>
              No messages yet. Say hello!
            </p>
          </div>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4, maxWidth: 700, width: '100%', alignSelf: 'center' }}>
            <AnimatePresence initial={false}>
              {grouped.map(item => {
                if (item.type === 'header') {
                  return (
                    <div key={item.key} style={{ textAlign: 'center', margin: '16px 0 8px' }}>
                      <span style={{
                        padding: '4px 12px', borderRadius: 100,
                        background: 'rgba(255,255,255,0.06)',
                        fontSize: 11, color: 'rgba(255,255,255,0.4)', fontWeight: 600,
                      }}>
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
                    style={{
                      display: 'flex',
                      justifyContent: isMine ? 'flex-end' : 'flex-start',
                      marginBottom: 2,
                    }}
                  >
                    <div style={{ maxWidth: '72%' }}>
                      <div style={{
                        padding: '10px 14px', borderRadius: isMine ? '18px 18px 4px 18px' : '18px 18px 18px 4px',
                        background: isMine
                          ? 'linear-gradient(135deg, #22C55E, #16A34A)'
                          : 'rgba(255,255,255,0.07)',
                        color: isMine ? '#0A0A0A' : 'white',
                        fontSize: 14, lineHeight: 1.5, wordBreak: 'break-word',
                        border: isMine ? 'none' : '1px solid rgba(255,255,255,0.08)',
                      }}>
                        {item.content}
                      </div>
                      <div style={{
                        fontSize: 10, color: 'rgba(255,255,255,0.3)',
                        marginTop: 3,
                        textAlign: isMine ? 'right' : 'left',
                        paddingLeft: isMine ? 0 : 4,
                        paddingRight: isMine ? 4 : 0,
                      }}>
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
      <div style={{
        position: 'fixed', bottom: 0, left: 0, right: 0,
        padding: '12px 16px 20px',
        background: 'rgba(10,10,10,0.95)',
        backdropFilter: 'blur(24px)',
        WebkitBackdropFilter: 'blur(24px)',
        borderTop: '1px solid rgba(255,255,255,0.07)',
      }}>
        <div style={{
          maxWidth: 700, margin: '0 auto',
          display: 'flex', gap: 10, alignItems: 'flex-end',
        }}>
          <textarea
            ref={inputRef}
            value={input}
            onChange={e => {
              setInput(e.target.value);
              // Auto-resize
              e.target.style.height = 'auto';
              e.target.style.height = Math.min(e.target.scrollHeight, 120) + 'px';
            }}
            onKeyDown={handleKeyDown}
            placeholder="Type a message…"
            rows={1}
            style={{
              flex: 1, padding: '12px 16px', borderRadius: 18,
              background: 'rgba(255,255,255,0.07)',
              border: '1px solid rgba(255,255,255,0.1)',
              color: 'white', fontSize: 14, outline: 'none',
              fontFamily: 'inherit', resize: 'none', lineHeight: 1.5,
              maxHeight: 120, overflowY: 'auto',
              transition: 'border-color 0.2s',
            }}
            onFocus={e => e.target.style.borderColor = 'rgba(34,197,94,0.4)'}
            onBlur={e => e.target.style.borderColor = 'rgba(255,255,255,0.1)'}
          />
          <button
            onClick={handleSend}
            disabled={!input.trim() || sending}
            style={{
              width: 46, height: 46, borderRadius: 16, flexShrink: 0,
              background: input.trim() && !sending ? '#22C55E' : 'rgba(255,255,255,0.08)',
              border: 'none', cursor: input.trim() && !sending ? 'pointer' : 'not-allowed',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              transition: 'all 0.2s',
            }}
          >
            <Send size={18} color={input.trim() && !sending ? '#0A0A0A' : 'rgba(255,255,255,0.3)'} />
          </button>
        </div>
      </div>
    </div>
  );
}

// ─── Shared styles ───────────────────────────────────────────────────────────

const iconBtn = {
  width: 38, height: 38, borderRadius: 12, flexShrink: 0,
  background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
  color: 'white', cursor: 'pointer',
  display: 'flex', alignItems: 'center', justifyContent: 'center',
};

const errorBox = {
  background: 'rgba(239,68,68,0.1)', color: '#EF4444',
  padding: '10px 14px', borderRadius: 12, fontSize: 13,
  fontWeight: 500, border: '1px solid rgba(239,68,68,0.2)',
};

const spinnerStyle = {
  width: 32, height: 32, borderRadius: '50%',
  border: '3px solid rgba(255,255,255,0.08)',
  borderTopColor: '#22C55E',
  animation: 'spin 0.8s linear infinite',
};
