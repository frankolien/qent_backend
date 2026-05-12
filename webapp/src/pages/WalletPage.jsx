import { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { motion, AnimatePresence, useInView } from 'framer-motion';
import {
  Wallet, ArrowDownLeft, ArrowUpRight, TrendingUp, Clock,
  DollarSign, ArrowLeft, CheckCircle, Download,
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
  credit:      { label: 'Credit',      color: '#22C55E', bg: 'rgba(34,197,94,0.1)',  icon: ArrowDownLeft, sign: '+' },
  debit:       { label: 'Debit',       color: '#EF4444', bg: 'rgba(239,68,68,0.1)',  icon: ArrowUpRight,  sign: '-' },
  booking:     { label: 'Booking',     color: '#3B82F6', bg: 'rgba(59,130,246,0.1)', icon: DollarSign,    sign: '-' },
  earning:     { label: 'Earning',     color: '#22C55E', bg: 'rgba(34,197,94,0.1)',  icon: ArrowDownLeft, sign: '+' },
  withdrawal:  { label: 'Withdrawal',  color: '#F59E0B', bg: 'rgba(245,158,11,0.1)', icon: ArrowUpRight,  sign: '-' },
  refund:      { label: 'Refund',      color: '#8B5CF6', bg: 'rgba(139,92,246,0.1)', icon: ArrowDownLeft, sign: '+' },
  platform_fee:{ label: 'Platform Fee',color: '#6B7280', bg: 'rgba(107,114,128,0.1)',icon: DollarSign,    sign: '-' },
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
  const canWithdraw = wallet?.balance > 0;

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      className="max-w-[800px] mx-auto pt-[100px] px-6 pb-20"
    >
      <button
        onClick={() => navigate(-1)}
        className="flex items-center justify-center w-10 h-10 rounded-xl mb-5 bg-white/[0.06] border border-white/[0.08] text-white cursor-pointer"
      >
        <ArrowLeft size={18} />
      </button>

      <div className="flex items-center gap-2.5 mb-8">
        <Wallet size={24} color="var(--accent)" />
        <h1 className="text-[30px] font-black tracking-tighter m-0">Wallet</h1>
      </div>

      {error && (
        <div className="bg-red-500/10 text-red-500 px-4 py-3 rounded-2xl text-[13px] font-medium mb-4 border border-red-500/20 flex items-center gap-2">
          {error}
        </div>
      )}
      {success && (
        <motion.div
          initial={{ opacity: 0, y: -8 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-accent/10 text-accent px-4 py-3 rounded-2xl text-[13px] font-medium border border-accent/20 flex items-center gap-2 mb-4"
        >
          <CheckCircle size={15} /> {success}
        </motion.div>
      )}

      {loading ? (
        <div className="flex flex-col items-center py-20 gap-4">
          <div className="spinner" />
          <p className="text-white/40 text-sm">Loading wallet…</p>
        </div>
      ) : (
        <>
          {/* Balance card */}
          <FadeIn>
            <div
              className="p-8 rounded-[28px] mb-4 border border-accent/25 relative overflow-hidden"
              style={{ background: 'linear-gradient(135deg, #0D1F13 0%, #0A0A0A 100%)' }}
            >
              <div
                className="absolute -top-[60px] -right-[60px] w-[200px] h-[200px] rounded-full pointer-events-none"
                style={{ background: 'radial-gradient(circle, rgba(34,197,94,0.12) 0%, transparent 70%)' }}
              />

              <p className="text-white/50 text-xs font-bold uppercase tracking-[1.5px] mb-2.5">Available Balance</p>
              <div className="text-5xl font-black text-accent tracking-[-2px] mb-1.5">
                {fmtMoney(wallet?.balance || 0)}
              </div>
              {pendingBalance > 0 && (
                <div className="flex items-center gap-1 text-white/40 text-[13px]">
                  <Clock size={13} /> {fmtMoney(pendingBalance)} pending
                </div>
              )}

              <button
                onClick={handleWithdraw}
                disabled={withdrawing || !canWithdraw}
                className={`mt-6 inline-flex items-center gap-2 px-6 py-[13px] rounded-2xl border-0 text-sm font-bold transition-all duration-200 ${
                  canWithdraw ? 'bg-accent text-black' : 'bg-white/[0.08] text-white/30'
                } ${
                  (withdrawing || !canWithdraw) ? 'cursor-not-allowed' : 'cursor-pointer'
                } ${withdrawing ? 'opacity-70' : 'opacity-100'}`}
              >
                <Download size={16} />
                {withdrawing ? 'Processing…' : 'Withdraw to Bank'}
              </button>
            </div>
          </FadeIn>

          {/* Stats row */}
          <FadeIn delay={0.1}>
            <div className="grid grid-cols-3 gap-3 mb-8">
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

          {/* Tabs */}
          <div className="flex gap-1 mb-5 bg-white/[0.04] rounded-2xl p-1">
            {TABS.map(t => (
              <button
                key={t}
                onClick={() => setTab(t)}
                className={`flex-1 p-2.5 rounded-[11px] text-[13px] font-bold border-0 cursor-pointer transition-all duration-200 ${
                  tab === t ? 'bg-white/10 text-white' : 'bg-transparent text-white/40'
                }`}
              >
                {t}
                {tab !== t && (
                  <span className="text-[11px] text-white/30 ml-1">
                    ({t === 'Transactions' ? transactions.length : earnings.length})
                  </span>
                )}
              </button>
            ))}
          </div>

          {/* List */}
          <AnimatePresence mode="wait">
            {tab === 'Transactions' ? (
              <motion.div key="tx" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -6 }} transition={{ duration: 0.25 }}>
                {transactions.length === 0 ? (
                  <EmptyList icon={<DollarSign size={28} color="rgba(255,255,255,0.15)" />} text="No transactions yet" />
                ) : (
                  <div className="flex flex-col gap-0.5">
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
                  <div className="flex flex-col gap-0.5">
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
      className={`flex items-center gap-3.5 px-4 py-3.5 rounded-2xl cursor-default transition-colors duration-150 ${
        hovered ? 'bg-white/[0.04]' : 'bg-transparent'
      }`}
    >
      <div
        className="w-[42px] h-[42px] rounded-2xl flex-shrink-0 flex items-center justify-center"
        style={{ background: meta.bg }}
      >
        <TxIcon size={18} color={meta.color} />
      </div>
      <div className="flex-1 min-w-0">
        <div className="text-sm font-semibold text-white mb-[3px]">
          {tx.description || meta.label}
        </div>
        <div className="text-xs text-white/40">
          {fmtDate(tx.created_at)} {fmtTime(tx.created_at) && '· ' + fmtTime(tx.created_at)}
        </div>
      </div>
      <div className="text-right flex-shrink-0">
        <div className={`text-base font-extrabold ${isCredit ? 'text-accent' : 'text-red-500'}`}>
          {meta.sign}{fmtMoney(tx.amount)}
        </div>
        {tx.status && (
          <div className="text-[11px] text-white/35 mt-0.5 capitalize">
            {tx.status}
          </div>
        )}
      </div>
    </motion.div>
  );
}

function EarningRow({ earning, index }) {
  const [hovered, setHovered] = useState(false);

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.04 }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      className={`flex items-center gap-3.5 px-4 py-3.5 rounded-2xl cursor-default transition-colors duration-150 ${
        hovered ? 'bg-white/[0.04]' : 'bg-transparent'
      }`}
    >
      <div className="w-[42px] h-[42px] rounded-2xl flex-shrink-0 flex items-center justify-center bg-accent/10">
        <TrendingUp size={18} color="#22C55E" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="text-sm font-semibold text-white mb-[3px]">
          {earning.description || earning.car_name || 'Trip earnings'}
        </div>
        <div className="text-xs text-white/40 flex gap-2">
          <span>{fmtDate(earning.completed_at || earning.created_at || earning.date)}</span>
          {earning.booking_id && (
            <span className="text-white/25">
              #{String(earning.booking_id).slice(0, 8).toUpperCase()}
            </span>
          )}
          {earning.renter_name && (
            <span className="text-white/30">{earning.renter_name}</span>
          )}
        </div>
      </div>
      <div className="text-right flex-shrink-0">
        <div className="text-base font-extrabold text-accent">
          +{fmtMoney(earning.earned || earning.amount)}
        </div>
      </div>
    </motion.div>
  );
}

function StatCard({ icon, label, value, color, bg, border }) {
  return (
    <div
      className="px-[18px] py-4 rounded-[18px]"
      style={{ background: bg, border: `1px solid ${border}` }}
    >
      <div className="flex items-center gap-1.5 mb-2">
        {icon}
        <span className="text-[11px] font-bold text-white/40 uppercase tracking-wider">{label}</span>
      </div>
      <div className="text-xl font-extrabold" style={{ color }}>{value}</div>
    </div>
  );
}

function EmptyList({ icon, text }) {
  return (
    <div className="text-center px-6 py-[60px]">
      <div className="w-[60px] h-[60px] rounded-[18px] bg-white/[0.04] flex items-center justify-center mx-auto mb-3">
        {icon}
      </div>
      <p className="text-white/35 text-sm">{text}</p>
    </div>
  );
}

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
