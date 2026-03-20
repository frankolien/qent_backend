import { useState, useEffect, useRef } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { motion, useInView, useScroll, useTransform } from 'framer-motion';
import { Search, ArrowRight, Shield, Clock, Wallet, Star, MapPin, ChevronRight } from 'lucide-react';
import CarCard from '../components/CarCard';
import Logo from '../components/Logo';
import { getHomepage, searchCars } from '../utils/api';

// Hero car images (Unsplash)
const HERO_CARS = [
  'https://images.unsplash.com/photo-1544636331-e26879cd4d9b?w=1400&q=80',
  'https://images.unsplash.com/photo-1503376780353-7e6692767b70?w=1400&q=80',
  'https://images.unsplash.com/photo-1492144534655-ae79c964c9d7?w=1400&q=80',
];

export default function Home() {
  const [sections, setSections] = useState(null);
  const [loading, setLoading] = useState(true);
  const [heroIdx, setHeroIdx] = useState(0);
  const [searchQuery, setSearchQuery] = useState('');
  const navigate = useNavigate();

  useEffect(() => {
    // Try to get user location for nearby results
    if (navigator.geolocation) {
      navigator.geolocation.getCurrentPosition(
        (pos) => {
          getHomepage({ latitude: pos.coords.latitude, longitude: pos.coords.longitude })
            .then(r => setSections(r.data)).catch(console.error).finally(() => setLoading(false));
        },
        () => {
          // Location denied — fetch without coords
          getHomepage().then(r => setSections(r.data)).catch(console.error).finally(() => setLoading(false));
        },
        { timeout: 5000 }
      );
    } else {
      getHomepage().then(r => setSections(r.data)).catch(console.error).finally(() => setLoading(false));
    }
  }, []);

  useEffect(() => {
    const t = setInterval(() => setHeroIdx(i => (i + 1) % HERO_CARS.length), 5000);
    return () => clearInterval(t);
  }, []);

  const handleSearch = (e) => {
    e.preventDefault();
    if (searchQuery.trim()) navigate(`/search?q=${encodeURIComponent(searchQuery)}`);
  };

  return (
    <div>
      {/* ── HERO ────────────────────────── */}
      <section style={{ position: 'relative', height: '100vh', overflow: 'hidden' }}>
        {/* Background images */}
        {HERO_CARS.map((url, i) => (
          <div key={i} style={{
            position: 'absolute', inset: 0, transition: 'opacity 1.5s ease',
            opacity: heroIdx === i ? 1 : 0,
          }}>
            <img src={url} alt="" style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
          </div>
        ))}

        {/* Overlay gradient */}
        <div style={{
          position: 'absolute', inset: 0,
          background: 'linear-gradient(180deg, rgba(0,0,0,0.6) 0%, rgba(0,0,0,0.4) 50%, rgba(10,10,10,1) 100%)',
        }} />

        {/* Content */}
        <div style={{ position: 'relative', zIndex: 10, maxWidth: 1280, margin: '0 auto', padding: '0 32px', height: '100%', display: 'flex', flexDirection: 'column', justifyContent: 'center' }}>
          <motion.div
            initial={{ opacity: 0, y: 40 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.2 }}
          >
            <div style={{ display: 'inline-flex', alignItems: 'center', gap: 8, padding: '8px 16px', background: 'rgba(34,197,94,0.15)', border: '1px solid rgba(34,197,94,0.3)', borderRadius: 100, marginBottom: 28 }}>
              <div style={{ width: 6, height: 6, borderRadius: '50%', background: 'var(--accent)', animation: 'pulse 2s infinite' }} />
              <span style={{ fontSize: 13, fontWeight: 600, color: 'var(--accent)' }}>Now live in Nigeria</span>
            </div>

            <h1 style={{ fontSize: 'clamp(40px, 6vw, 76px)', fontWeight: 900, lineHeight: 1, letterSpacing: -3, maxWidth: 700, marginBottom: 20 }}>
              Drive <span style={{ color: 'var(--accent)' }}>anything</span>,<br/>anywhere.
            </h1>
            <p style={{ fontSize: 18, color: 'rgba(255,255,255,0.6)', maxWidth: 500, lineHeight: 1.6, marginBottom: 36 }}>
              Rent cars from trusted hosts near you. Or list yours and earn. Nigeria's peer-to-peer car rental platform.
            </p>

            {/* Search bar */}
            <form onSubmit={handleSearch} style={{
              display: 'flex', maxWidth: 520, borderRadius: 20, overflow: 'hidden',
              background: 'rgba(255,255,255,0.08)', border: '1px solid rgba(255,255,255,0.1)',
              backdropFilter: 'blur(20px)',
            }}>
              <div style={{ flex: 1, display: 'flex', alignItems: 'center', padding: '0 20px' }}>
                <Search size={18} color="rgba(255,255,255,0.4)" />
                <input
                  value={searchQuery} onChange={e => setSearchQuery(e.target.value)}
                  placeholder="Where do you need a car?"
                  style={{ flex: 1, border: 'none', background: 'none', padding: '18px 14px', fontSize: 15, color: 'white', outline: 'none', fontFamily: 'inherit' }}
                />
              </div>
              <button type="submit" style={{
                padding: '0 28px', background: 'var(--accent)', border: 'none',
                fontSize: 14, fontWeight: 700, color: 'var(--black)', fontFamily: 'inherit',
              }}>Search</button>
            </form>

            {/* Stats */}
            <div style={{ display: 'flex', gap: 40, marginTop: 48 }}>
              <Stat value="1,000+" label="Cars listed" />
              <Stat value="85%" label="Host keeps" />
              <Stat value="24h" label="Avg payout" />
            </div>
          </motion.div>
        </div>

        {/* Scroll indicator */}
        <motion.div
          animate={{ y: [0, 8, 0] }}
          transition={{ repeat: Infinity, duration: 2 }}
          style={{ position: 'absolute', bottom: 40, left: '50%', transform: 'translateX(-50%)', zIndex: 10 }}
        >
          <div style={{ width: 24, height: 40, borderRadius: 12, border: '2px solid rgba(255,255,255,0.2)', display: 'flex', justifyContent: 'center', paddingTop: 8 }}>
            <div style={{ width: 3, height: 8, borderRadius: 2, background: 'rgba(255,255,255,0.4)' }} />
          </div>
        </motion.div>
      </section>

      {/* ── BRANDS ──────────────────────── */}
      <section style={{ padding: '80px 0 0', background: 'var(--black)' }}>
        <div style={{ maxWidth: 1280, margin: '0 auto', padding: '0 32px' }}>
          <FadeIn>
            <p style={{ textAlign: 'center', fontSize: 13, textTransform: 'uppercase', letterSpacing: 3, color: 'var(--gray-500)', marginBottom: 32 }}>Trusted brands available</p>
          </FadeIn>
          <div style={{ display: 'flex', justifyContent: 'center', gap: 48, flexWrap: 'wrap', opacity: 0.4 }}>
            {['Toyota', 'Mercedes-Benz', 'BMW', 'Lexus', 'Range Rover', 'Honda'].map(b => (
              <span key={b} style={{ fontSize: 18, fontWeight: 700, letterSpacing: -0.5 }}>{b}</span>
            ))}
          </div>
        </div>
      </section>

      {/* ── CAR SECTIONS ────────────────── */}
      <section id="cars" style={{ padding: '80px 0', background: 'var(--black)' }}>
        <div style={{ maxWidth: 1280, margin: '0 auto', padding: '0 32px' }}>
          {loading ? (
            <div style={{ textAlign: 'center', padding: 80, color: 'var(--gray-500)' }}>Loading cars...</div>
          ) : sections && (
            <>
              {sections.recommended?.length > 0 && <CarSection title="Recommended for You" subtitle="Based on your preferences" cars={sections.recommended} />}
              {sections.nearby?.length > 0 && <CarSection title="Nearby Cars" subtitle="Cars close to you" cars={sections.nearby} />}
              {sections.best_cars?.length > 0 && <CarSection title="Best Cars" subtitle="Top rated on Qent" cars={sections.best_cars} />}
              {sections.popular?.length > 0 && <CarSection title="Popular Cars" subtitle="Most booked this month" cars={sections.popular} />}
            </>
          )}
        </div>
      </section>

      {/* ── HOW IT WORKS ────────────────── */}
      <section id="how-it-works" style={{ padding: '100px 0', background: 'linear-gradient(180deg, var(--black) 0%, #0F0F0F 100%)' }}>
        <div style={{ maxWidth: 1280, margin: '0 auto', padding: '0 32px' }}>
          <FadeIn>
            <p style={{ fontSize: 13, textTransform: 'uppercase', letterSpacing: 3, color: 'var(--accent)', marginBottom: 12 }}>How it works</p>
            <h2 style={{ fontSize: 'clamp(28px, 4vw, 48px)', fontWeight: 900, letterSpacing: -2, marginBottom: 64 }}>Three steps to your ride.</h2>
          </FadeIn>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 24 }}>
            <StepCard num="01" icon={<Search size={28} />} title="Find a car" desc="Browse hundreds of cars near you. Filter by price, brand, location, and date." />
            <StepCard num="02" icon={<Shield size={28} />} title="Book securely" desc="Pay through Paystack. Every trip comes with a protection plan. Host approves instantly." />
            <StepCard num="03" icon={<ArrowRight size={28} />} title="Pick up & drive" desc="Meet the host, inspect the car, and hit the road. Return when you're done." />
          </div>
        </div>
      </section>

      {/* ── HOST CTA ────────────────────── */}
      <section id="host" style={{ padding: '100px 0', background: '#0F0F0F', position: 'relative', overflow: 'hidden' }}>
        {/* Glow */}
        <div style={{ position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%,-50%)', width: 600, height: 600, borderRadius: '50%', background: 'radial-gradient(circle, rgba(34,197,94,0.1) 0%, transparent 70%)' }} />

        <div style={{ maxWidth: 800, margin: '0 auto', padding: '0 32px', textAlign: 'center', position: 'relative', zIndex: 1 }}>
          <FadeIn>
            <p style={{ fontSize: 13, textTransform: 'uppercase', letterSpacing: 3, color: 'var(--accent)', marginBottom: 12 }}>For car owners</p>
            <h2 style={{ fontSize: 'clamp(32px, 5vw, 56px)', fontWeight: 900, letterSpacing: -2, marginBottom: 20 }}>Your car could be earning.</h2>
            <p style={{ fontSize: 18, color: 'var(--gray-500)', lineHeight: 1.7, marginBottom: 40 }}>
              The average Qent host earns up to {"\u20A6"}300,000/month. Your car sits idle most of the time — put it to work.
            </p>

            <div style={{ display: 'flex', justifyContent: 'center', gap: 48, marginBottom: 48 }}>
              <div><div style={{ fontSize: 36, fontWeight: 900, color: 'var(--accent)' }}>85%</div><div style={{ fontSize: 13, color: 'var(--gray-500)', marginTop: 4 }}>You keep per booking</div></div>
              <div><div style={{ fontSize: 36, fontWeight: 900 }}>{"\u20A6"}0</div><div style={{ fontSize: 13, color: 'var(--gray-500)', marginTop: 4 }}>To list your car</div></div>
              <div><div style={{ fontSize: 36, fontWeight: 900 }}>24h</div><div style={{ fontSize: 13, color: 'var(--gray-500)', marginTop: 4 }}>Average payout</div></div>
            </div>

            <Link to="/signup" style={{
              display: 'inline-flex', alignItems: 'center', gap: 8,
              padding: '16px 36px', background: 'white', color: 'var(--black)',
              borderRadius: 100, fontSize: 16, fontWeight: 700, transition: 'transform 0.2s',
            }}>
              Start earning <ArrowRight size={18} />
            </Link>
          </FadeIn>
        </div>
      </section>

      {/* ── FEATURES GRID ───────────────── */}
      <section style={{ padding: '100px 0', background: 'var(--black)' }}>
        <div style={{ maxWidth: 1280, margin: '0 auto', padding: '0 32px' }}>
          <FadeIn>
            <h2 style={{ fontSize: 'clamp(28px, 4vw, 48px)', fontWeight: 900, letterSpacing: -2, textAlign: 'center', marginBottom: 64 }}>Built different.</h2>
          </FadeIn>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 16 }}>
            <FeatureCard icon={<Shield size={24} color="var(--accent)" />} title="Verified users" desc="Every renter and host is ID-verified before they can book or list." />
            <FeatureCard icon={<Wallet size={24} color="var(--accent)" />} title="Instant payouts" desc="Hosts withdraw earnings to any Nigerian bank account. No delays." />
            <FeatureCard icon={<Clock size={24} color="var(--accent)" />} title="Book in minutes" desc="No paperwork, no agencies. Find a car and book in under 2 minutes." />
            <FeatureCard icon={<Star size={24} color="var(--accent)" />} title="Ratings & reviews" desc="Real reviews from real trips. Know who you're renting from." />
            <FeatureCard icon={<MapPin size={24} color="var(--accent)" />} title="Cars everywhere" desc="Lagos, Abuja, PH, Kano, and more cities coming soon." />
            <FeatureCard icon={<Search size={24} color="var(--accent)" />} title="Smart matching" desc="Personalized recommendations based on your preferences and history." />
          </div>
        </div>
      </section>

      {/* ── FOOTER ──────────────────────── */}
      <footer style={{ padding: '60px 0 40px', borderTop: '1px solid rgba(255,255,255,0.06)' }}>
        <div style={{ maxWidth: 1280, margin: '0 auto', padding: '0 32px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            <Logo variant="full" size={28} />
          </div>
          <p style={{ fontSize: 13, color: 'var(--gray-500)' }}>&copy; 2026 Qent. All rights reserved.</p>
          <div style={{ display: 'flex', gap: 24 }}>
            <a href="#" style={{ fontSize: 13, color: 'var(--gray-500)' }}>Privacy</a>
            <a href="#" style={{ fontSize: 13, color: 'var(--gray-500)' }}>Terms</a>
          </div>
        </div>
      </footer>

      <style>{`
        @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }
      `}</style>
    </div>
  );
}

// ─── Components ─────────────────────────

function Stat({ value, label }) {
  return (
    <div>
      <div style={{ fontSize: 24, fontWeight: 800, letterSpacing: -0.5 }}>{value}</div>
      <div style={{ fontSize: 13, color: 'rgba(255,255,255,0.4)', marginTop: 2 }}>{label}</div>
    </div>
  );
}

function CarSection({ title, subtitle, cars }) {
  return (
    <FadeIn>
      <div style={{ marginBottom: 64 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end', marginBottom: 24 }}>
          <div>
            <h2 style={{ fontSize: 28, fontWeight: 800, letterSpacing: -0.5, marginBottom: 4 }}>{title}</h2>
            {subtitle && <p style={{ color: 'var(--gray-500)', fontSize: 14 }}>{subtitle}</p>}
          </div>
          <span style={{ fontSize: 14, fontWeight: 600, color: 'var(--accent)', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: 4 }}>
            View all <ChevronRight size={16} />
          </span>
        </div>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: 20 }}>
          {cars.slice(0, 6).map(car => <CarCard key={car.id} car={car} />)}
        </div>
      </div>
    </FadeIn>
  );
}

function StepCard({ num, icon, title, desc }) {
  return (
    <FadeIn delay={Number(num) * 0.1}>
      <div style={{
        padding: 36, borderRadius: 24,
        background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.06)',
        transition: 'all 0.3s',
      }}
        onMouseEnter={e => { e.currentTarget.style.background = 'rgba(255,255,255,0.06)'; e.currentTarget.style.borderColor = 'rgba(34,197,94,0.2)'; }}
        onMouseLeave={e => { e.currentTarget.style.background = 'rgba(255,255,255,0.03)'; e.currentTarget.style.borderColor = 'rgba(255,255,255,0.06)'; }}
      >
        <div style={{ fontSize: 48, fontWeight: 900, color: 'rgba(255,255,255,0.06)', marginBottom: 20 }}>{num}</div>
        <div style={{ marginBottom: 16, color: 'var(--accent)' }}>{icon}</div>
        <h3 style={{ fontSize: 20, fontWeight: 700, marginBottom: 8 }}>{title}</h3>
        <p style={{ color: 'var(--gray-500)', fontSize: 14, lineHeight: 1.7 }}>{desc}</p>
      </div>
    </FadeIn>
  );
}

function FeatureCard({ icon, title, desc }) {
  return (
    <FadeIn>
      <div style={{
        padding: 32, borderRadius: 20,
        background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.06)',
        transition: 'all 0.3s',
      }}
        onMouseEnter={e => { e.currentTarget.style.background = 'rgba(255,255,255,0.06)'; }}
        onMouseLeave={e => { e.currentTarget.style.background = 'rgba(255,255,255,0.03)'; }}
      >
        <div style={{ width: 48, height: 48, borderRadius: 14, background: 'rgba(34,197,94,0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center', marginBottom: 20 }}>{icon}</div>
        <h3 style={{ fontSize: 17, fontWeight: 700, marginBottom: 8 }}>{title}</h3>
        <p style={{ color: 'var(--gray-500)', fontSize: 14, lineHeight: 1.6 }}>{desc}</p>
      </div>
    </FadeIn>
  );
}

function FadeIn({ children, delay = 0 }) {
  const ref = useRef(null);
  const inView = useInView(ref, { once: true, margin: '-60px' });
  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 30 }}
      animate={inView ? { opacity: 1, y: 0 } : {}}
      transition={{ duration: 0.6, delay, ease: 'easeOut' }}
    >
      {children}
    </motion.div>
  );
}
