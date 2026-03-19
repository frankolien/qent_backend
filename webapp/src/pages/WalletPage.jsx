import { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { motion, AnimatePresence, useInView } from 'framer-motion';
import {
  Wallet, ArrowDownLeft, ArrowUpRight, TrendingUp, Clock,
  DollarSign, ArrowLeft, AlertCircle, CheckCircle, XCircle,
  Download,
} from 'lucide-react';
import api, { getWallet, getTransactions, getEarnings } from '../utils/api';
import { useAuth } from '../hooks/useAuth';

const TABS = ['Transactions', 'Earnings'];

function fmtMoney(n) {
  return '₦' + Number(n || 0).toLocaleString();
}

function fmtDate(d) {
  if (!d) return '—';
  return new Date(d).toLocaleDateString('en-NG', { month: 'short', day: 'numeric', year: 'numeric' });
}

function fmtTime(d) {
  if (!d) return '';
  return new Date(d).toLocaleTimeString('en-NG', { hour: '2-digit', minute: '2-digit', hour12: true });
}

const TX_TYPE_META = {
  credit:      { label: 'Credit',      color: '#22C55E', bg: 'rgba(34,197,94,0.1)',  icon: ArrowDownLeft,   sign: '+' },
  debit:       { label: 'Debit',       color: '#EF4444', bg: 'rgba(239,68,68,0.1)',  icon: ArrowUpRight,    sign: '-' },
  booking:     { label: 'Booking',     color: '#3B82F6', bg: 'rgba(59,130,246,0.1)', icon: DollarSign,      sign: '-' },
  earning:     { label: 'Earning',     color: '#22C55E', bg: 'rgba(34,197,94,0.1)',  icon: ArrowDownLeft,   sign: '+' },
  withdrawal:  { label: 'Withdrawal',  color: '#F59E0B', bg: 'rgba(245,158,11,0.1)', icon: ArrowUpRight,    sign: '-' },
  refund:      { label: 'Refund',      color: '#8B5CF6', bg: 'rgba(139,92,246,0.1)', icon: ArrowDownLeft,   sign: '+' },
  platform_fee:{ label: 'Platform Fee',color: '#6B7280', bg: 'rgba(107,114,128,0.1)',icon: DollarSign,      sign: '-' },
};

function getTxMeta(tx) {
  const type = (tx.transaction_type || tx.type || '').toLowerCase();
  return TX_TYPE_META[type] || { label: type || 'Transaction', color: '#6B7280', bg: 'rgba(107,114,128,0.1)', icon: DollarSign, sign: '' };
}

export default function WalletPage() {
  const { user } = useAuth();
  const navigate = useNavigate();
  const [wallet, setWallet] = useState(null);
  const [transactions, setTransactions] = useState([]);
  const [earnings, setEarnings] = useState([]);
  const [earningsStats, setEarningsStats] = useState(null);
  const [tab, setTab] = useState('Transactions');
  const [loading, setLoading] = useState(true);
  const [withdrawing, setWithdrawing] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');

  useEffect(() => {
    if (!user) { navigate('/login'); return; }
    const load = async () => {
      try {
        const [walletRes, txRes, earnRes] = await Promise.allSettled([
          getWallet(),
          getTransactions(),
          getEarnings(),
        ]);
        if (walletRes.status === 'fulfilled') setWallet(walletRes.value.data);
        if (txRes.status === 'fulfilled') {
          const txData = txRes.value.data;
          setTransactions(Array.isArray(txData) ? txData : txData?.transactions || []);
        }
        if (earnRes.status === 'fulfilled') {
          const earnData = earnRes.value.data;
          setEarningsStats(earnData);
          setEarnings(earnData?.recent_earnings || []);
        }
      } catch {
        setError('Failed to load wallet data');
      }
      setLoading(false);
    };
    load();
  }, [user]);

  const handleWithdraw = async () => {
    if (!wallet?.balance || wallet.balance <= 0) {
      setError('No balance available for withdrawal');
      return;
    }
    setWithdrawing(true);
    setError('');
    try {
      await api.post('/payments/wallet/withdraw', { amount: wallet.balance });
      setSuccess('Withdrawal initiated! Your funds will arrive within 24 hours.');
      setTimeout(() => setSuccess(''), 5000);
      const r = await getWallet();
      setWallet(r.data);
    } catch (e) {
      setError(e.response?.data?.error || 'Withdrawal failed. Please try again.');
    }
    setWithdrawing(false);
  };

  const pendingBalance = earningsStats?.pending_earnings || 0;
  const totalEarned = earningsStats?.total_earned || 0;
  const totalPlatformFee = transactions
    .filter(t => (t.transaction_type || t.type || '').toLowerCase() === 'platform_fee')
    .reduce((sum, t) => sum + (t.amount || 0), 0);
  const tripCount = earningsStats?.completed_trips || 0;

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      style={{ maxWidth: 800, margin: '0 auto', padding: '100px 24px 80px' }}
    >
      {/* Header */}
      <button onClick={() => navigate(-1)} style={backBtn}>
        <ArrowLeft size={18} />
      </button>

      <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 32 }}>
        <Wallet size={24} color="var(--accent)" />
        <h1 style={{ fontSize: 30, fontWeight: 900, letterSpacing: -1, margin: 0 }}>Wallet</h1>
      </div>

      {error && <div style={errorBox}>{error}</div>}
      {success && (
        <motion.div
          initial={{ opacity: 0, y: -8 }}
          animate={{ opacity: 1, y: 0 }}
          style={{ ...successBox, marginBottom: 16 }}
        >
          <CheckCircle size={15} /> {success}
        </motion.div>
      )}

      {loading ? (
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '80px 0', gap: 16 }}>
          <div style={spinnerStyle} />
          <p style={{ color: 'rgba(255,255,255,0.4)', fontSize: 14 }}>Loading wallet…</p>
        </div>
      ) : (
        <>
          {/* ── Balance card ─────────────────────────────────── */}
          <FadeIn>
            <div style={{
              padding: '32px', borderRadius: 28, marginBottom: 16,
              background: 'linear-gradient(135deg, #0D1F13 0%, #0A0A0A 100%)',
              border: '1px solid rgba(34,197,94,0.25)',
              position: 'relative', overflow: 'hidden',
            }}>
              {/* Decorative glow */}
              <div style={{
                position: 'absolute', top: -60, right: -60, width: 200, height: 200,
                borderRadius: '50%', background: 'radial-gradient(circle, rgba(34,197,94,0.12) 0%, transparent 70%)',
                pointerEvents: 'none',
              }} />

              <p style={{ color: 'rgba(255,255,255,0.5)', fontSize: 12, fontWeight: 700, textTransform: 'uppercase', letterSpacing: 1.5, marginBottom: 10 }}>Available Balance</p>
              <div style={{ fontSize: 48, fontWeight: 900, color: '#22C55E', letterSpacing: -2, marginBottom: 6 }}>
                {fmtMoney(wallet?.balance || 0)}
              </div>
              {pendingBalance > 0 && (
                <div style={{ display: 'flex', alignItems: 'center', gap: 5, color: 'rgba(255,255,255,0.4)', fontSize: 13 }}>
                  <Clock size={13} /> {fmtMoney(pendingBalance)} pending
                </div>
              )}

              <button
                onClick={handleWithdraw}
                disabled={withdrawing || !wallet?.balance || wallet.balance <= 0}
                style={{
                  marginTop: 24, display: 'inline-flex', alignItems: 'center', gap: 8,
                  padding: '13px 24px', borderRadius: 14,
                  background: wallet?.balance > 0 ? '#22C55E' : 'rgba(255,255,255,0.08)',
                  color: wallet?.balance > 0 ? '#0A0A0A' : 'rgba(255,255,255,0.3)',
                  border: 'none', fontSize: 14, fontWeight: 700,
                  cursor: (withdrawing || !wallet?.balance || wallet.balance <= 0) ? 'not-allowed' : 'pointer',
                  fontFamily: 'inherit', transition: 'all 0.2s',
                  opacity: withdrawing ? 0.7 : 1,
                }}
              >
                <Download size={16} />
                {withdrawing ? 'Processing…' : 'Withdraw to Bank'}
              </button>
            </div>
          </FadeIn>

          {/* ── Stats row ─────────────────────────────────────── */}
          <FadeIn delay={0.1}>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 12, marginBottom: 32 }}>
              <StatCard
                icon={<Clock size={16} color="#F59E0B" />}
                label="Pending"
                value={fmtMoney(pendingBalance)}
                color="#F59E0B"
                bg="rgba(245,158,11,0.08)"
                border="rgba(245,158,11,0.15)"
              />
              <StatCard
                icon={<TrendingUp size={16} color="#22C55E" />}
                label={`${tripCount} Trip${tripCount !== 1 ? 's' : ''}`}
                value={fmtMoney(totalEarned)}
                color="#22C55E"
                bg="rgba(34,197,94,0.08)"
                border="rgba(34,197,94,0.15)"
              />
              <StatCard
                icon={<DollarSign size={16} color="#6B7280" />}
                label="Platform Fee"
                value={fmtMoney(totalPlatformFee)}
                color="#6B7280"
                bg="rgba(107,114,128,0.08)"
                border="rgba(107,114,128,0.15)"
              />
            </div>
          </FadeIn>

          {/* ── Tabs ──────────────────────────────────────────── */}
          <div style={{ display: 'flex', gap: 4, marginBottom: 20, background: 'rgba(255,255,255,0.04)', borderRadius: 14, padding: 4 }}>
            {TABS.map(t => (
              <button
                key={t}
                onClick={() => setTab(t)}
                style={{
                  flex: 1, padding: '10px', borderRadius: 11, fontSize: 13, fontWeight: 700,
                  background: tab === t ? 'rgba(255,255,255,0.1)' : 'transparent',
                  color: tab === t ? 'white' : 'rgba(255,255,255,0.4)',
                  border: 'none', cursor: 'pointer', fontFamily: 'inherit',
                  transition: 'all 0.2s',
                }}
              >
                {t} {tab !== t && (
                  <span style={{ fontSize: 11, color: 'rgba(255,255,255,0.3)' }}>
                    ({t === 'Transactions' ? transactions.length : earnings.length})
                  </span>
                )}
              </button>
            ))}
          </div>

          {/* ── List ──────────────────────────────────────────── */}
          <AnimatePresence mode="wait">
            {tab === 'Transactions' ? (
              <motion.div key="tx" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -6 }} transition={{ duration: 0.25 }}>
                {transactions.length === 0 ? (
                  <EmptyList icon={<DollarSign size={28} color="rgba(255,255,255,0.15)" />} text="No transactions yet" />
                ) : (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                    {transactions.map((tx, i) => (
                      <TxRow key={tx.id || i} tx={tx} index={i} />
                    ))}
                  </div>
                )}
              </motion.div>
            ) : (
              <motion.div key="earn" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -6 }} transition={{ duration: 0.25 }}>
                {earnings.length === 0 ? (
                  <EmptyList icon={<TrendingUp size={28} color="rgba(255,255,255,0.15)" />} text="No earnings yet" />
                ) : (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                    {earnings.map((e, i) => (
                      <EarningRow key={e.id || i} earning={e} index={i} />
                    ))}
                  </div>
                )}
              </motion.div>
            )}
          </AnimatePresence>
        </>
      )}
    </motion.div>
  );
}

// ─── Transaction Row ─────────────────────────────────────────────────────────

function TxRow({ tx, index }) {
  const [hovered, setHovered] = useState(false);
  const meta = getTxMeta(tx);
  const TxIcon = meta.icon;
  const isCredit = meta.sign === '+';

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.04 }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: 'flex', alignItems: 'center', gap: 14, padding: '14px 16px',
        borderRadius: 14, cursor: 'default',
        background: hovered ? 'rgba(255,255,255,0.04)' : 'transparent',
        transition: 'background 0.15s',
      }}
    >
      <div style={{
        width: 42, height: 42, borderRadius: 14, flexShrink: 0,
        background: meta.bg, display: 'flex', alignItems: 'center', justifyContent: 'center',
      }}>
        <TxIcon size={18} color={meta.color} />
      </div>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ fontSize: 14, fontWeight: 600, color: 'white', marginBottom: 3 }}>
          {tx.description || meta.label}
        </div>
        <div style={{ fontSize: 12, color: 'rgba(255,255,255,0.4)' }}>
          {fmtDate(tx.created_at)} {fmtTime(tx.created_at) && '· ' + fmtTime(tx.created_at)}
        </div>
      </div>
      <div style={{ textAlign: 'right', flexShrink: 0 }}>
        <div style={{
          fontSize: 16, fontWeight: 800,
          color: isCredit ? '#22C55E' : '#EF4444',
        }}>
          {meta.sign}{fmtMoney(tx.amount)}
        </div>
        {tx.status && (
          <div style={{ fontSize: 11, color: 'rgba(255,255,255,0.35)', marginTop: 2, textTransform: 'capitalize' }}>
            {tx.status}
          </div>
        )}
      </div>
    </motion.div>
  );
}

// ─── Earning Row ─────────────────────────────────────────────────────────────

function EarningRow({ earning, index }) {
  const [hovered, setHovered] = useState(false);

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.04 }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: 'flex', alignItems: 'center', gap: 14, padding: '14px 16px',
        borderRadius: 14, cursor: 'default',
        background: hovered ? 'rgba(255,255,255,0.04)' : 'transparent',
        transition: 'background 0.15s',
      }}
    >
      <div style={{
        width: 42, height: 42, borderRadius: 14, flexShrink: 0,
        background: 'rgba(34,197,94,0.1)',
        display: 'flex', alignItems: 'center', justifyContent: 'center',
      }}>
        <TrendingUp size={18} color="#22C55E" />
      </div>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ fontSize: 14, fontWeight: 600, color: 'white', marginBottom: 3 }}>
          {earning.description || earning.car_name || 'Trip earnings'}
        </div>
        <div style={{ fontSize: 12, color: 'rgba(255,255,255,0.4)', display: 'flex', gap: 8 }}>
          <span>{fmtDate(earning.completed_at || earning.created_at || earning.date)}</span>
          {earning.booking_id && (
            <span style={{ color: 'rgba(255,255,255,0.25)' }}>
              #{String(earning.booking_id).slice(0, 8).toUpperCase()}
            </span>
          )}
          {earning.renter_name && (
            <span style={{ color: 'rgba(255,255,255,0.3)' }}>
              {earning.renter_name}
            </span>
          )}
        </div>
      </div>
      <div style={{ textAlign: 'right', flexShrink: 0 }}>
        <div style={{ fontSize: 16, fontWeight: 800, color: '#22C55E' }}>
          +{fmtMoney(earning.earned || earning.amount)}
        </div>
      </div>
    </motion.div>
  );
}

// ─── Stat Card ───────────────────────────────────────────────────────────────

function StatCard({ icon, label, value, color, bg, border }) {
  return (
    <div style={{
      padding: '16px 18px', borderRadius: 18,
      background: bg, border: `1px solid ${border}`,
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 8 }}>
        {icon}
        <span style={{ fontSize: 11, fontWeight: 700, color: 'rgba(255,255,255,0.4)', textTransform: 'uppercase', letterSpacing: 0.5 }}>{label}</span>
      </div>
      <div style={{ fontSize: 20, fontWeight: 800, color }}>{value}</div>
    </div>
  );
}

// ─── Empty List ───────────────────────────────────────────────────────────────

function EmptyList({ icon, text }) {
  return (
    <div style={{ textAlign: 'center', padding: '60px 24px' }}>
      <div style={{
        width: 60, height: 60, borderRadius: 18,
        background: 'rgba(255,255,255,0.04)', display: 'flex',
        alignItems: 'center', justifyContent: 'center', margin: '0 auto 12px',
      }}>
        {icon}
      </div>
      <p style={{ color: 'rgba(255,255,255,0.35)', fontSize: 14 }}>{text}</p>
    </div>
  );
}

// ─── FadeIn helper ───────────────────────────────────────────────────────────

function FadeIn({ children, delay = 0 }) {
  const ref = useRef(null);
  const inView = useInView(ref, { once: true, margin: '-40px' });
  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 20 }}
      animate={inView ? { opacity: 1, y: 0 } : {}}
      transition={{ duration: 0.5, delay, ease: 'easeOut' }}
    >
      {children}
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
  fontWeight: 500, marginBottom: 16, border: '1px solid rgba(239,68,68,0.2)',
  display: 'flex', alignItems: 'center', gap: 8,
};

const successBox = {
  background: 'rgba(34,197,94,0.1)', color: '#22C55E',
  padding: '12px 16px', borderRadius: 14, fontSize: 13,
  fontWeight: 500, border: '1px solid rgba(34,197,94,0.2)',
  display: 'flex', alignItems: 'center', gap: 8,
};

const spinnerStyle = {
  width: 36, height: 36, borderRadius: '50%',
  border: '3px solid rgba(255,255,255,0.08)',
  borderTopColor: '#22C55E',
  animation: 'spin 0.8s linear infinite',
};
