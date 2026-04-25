import { useState, useEffect, useRef } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { motion, useInView } from 'framer-motion';
import { Search, ArrowRight, Shield, Clock, Wallet, Star, MapPin, ChevronRight } from 'lucide-react';
import CarCard from '../components/CarCard';
import Logo from '../components/Logo';
import { getHomepage } from '../utils/api';

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
    if (navigator.geolocation) {
      navigator.geolocation.getCurrentPosition(
        (pos) => {
          getHomepage({ latitude: pos.coords.latitude, longitude: pos.coords.longitude })
            .then(r => setSections(r.data)).catch(console.error).finally(() => setLoading(false));
        },
        () => {
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
      {/* HERO */}
      <section className="relative h-screen overflow-hidden">
        {HERO_CARS.map((url, i) => (
          <div
            key={i}
            className={`absolute inset-0 transition-opacity duration-[1500ms] ease-out ${
              heroIdx === i ? 'opacity-100' : 'opacity-0'
            }`}
          >
            <img src={url} alt="" className="w-full h-full object-cover" />
          </div>
        ))}

        {/* Overlay gradient */}
        <div
          className="absolute inset-0"
          style={{ background: 'linear-gradient(180deg, rgba(0,0,0,0.6) 0%, rgba(0,0,0,0.4) 50%, rgba(10,10,10,1) 100%)' }}
        />

        {/* Content */}
        <div className="relative z-10 max-w-[1280px] mx-auto px-8 h-full flex flex-col justify-center">
          <motion.div
            initial={{ opacity: 0, y: 40 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.8, delay: 0.2 }}
          >
            <div className="inline-flex items-center gap-2 px-4 py-2 bg-accent/15 border border-accent/30 rounded-full mb-7">
              <div className="w-1.5 h-1.5 rounded-full bg-accent animate-pulse-soft" />
              <span className="text-[13px] font-semibold text-accent">Now live in Nigeria</span>
            </div>

            <h1
              className="font-black leading-none max-w-[700px] mb-5"
              style={{ fontSize: 'clamp(40px, 6vw, 76px)', letterSpacing: '-3px' }}
            >
              Drive <span className="text-accent">anything</span>,<br />anywhere.
            </h1>
            <p className="text-lg text-white/60 max-w-[500px] leading-relaxed mb-9">
              Rent cars from trusted hosts near you. Or list yours and earn. Nigeria's peer-to-peer car rental platform.
            </p>

            {/* Search bar */}
            <form
              onSubmit={handleSearch}
              className="flex max-w-[520px] rounded-[20px] overflow-hidden bg-white/[0.08] border border-white/10 backdrop-blur-glass"
            >
              <div className="flex-1 flex items-center px-5">
                <Search size={18} color="rgba(255,255,255,0.4)" />
                <input
                  value={searchQuery}
                  onChange={e => setSearchQuery(e.target.value)}
                  placeholder="Where do you need a car?"
                  className="flex-1 border-0 bg-transparent px-3.5 py-[18px] text-[15px] text-white outline-none"
                />
              </div>
              <button
                type="submit"
                className="px-7 bg-accent border-0 text-sm font-bold text-black"
              >
                Search
              </button>
            </form>

            {/* Stats */}
            <div className="flex gap-10 mt-12">
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
          className="absolute bottom-10 left-1/2 -translate-x-1/2 z-10"
        >
          <div className="w-6 h-10 rounded-xl border-2 border-white/20 flex justify-center pt-2">
            <div className="w-[3px] h-2 rounded bg-white/40" />
          </div>
        </motion.div>
      </section>

      {/* BRANDS */}
      <section className="pt-20 bg-black">
        <div className="max-w-[1280px] mx-auto px-8">
          <FadeIn>
            <p className="text-center text-[13px] uppercase tracking-[3px] text-gray-500 mb-8">
              Trusted brands available
            </p>
          </FadeIn>
          <div className="flex justify-center gap-12 flex-wrap opacity-40">
            {['Toyota', 'Mercedes-Benz', 'BMW', 'Lexus', 'Range Rover', 'Honda'].map(b => (
              <span key={b} className="text-lg font-bold tracking-tight">{b}</span>
            ))}
          </div>
        </div>
      </section>

      {/* CAR SECTIONS */}
      <section id="cars" className="py-20 bg-black">
        <div className="max-w-[1280px] mx-auto px-8">
          {loading ? (
            <div className="text-center py-20 text-gray-500">Loading cars...</div>
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

      {/* HOW IT WORKS */}
      <section
        id="how-it-works"
        className="py-[100px]"
        style={{ background: 'linear-gradient(180deg, #0A0A0A 0%, #0F0F0F 100%)' }}
      >
        <div className="max-w-[1280px] mx-auto px-8">
          <FadeIn>
            <p className="text-[13px] uppercase tracking-[3px] text-accent mb-3">How it works</p>
            <h2 className="font-black mb-16" style={{ fontSize: 'clamp(28px, 4vw, 48px)', letterSpacing: '-2px' }}>
              Three steps to your ride.
            </h2>
          </FadeIn>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <StepCard num="01" icon={<Search size={28} />} title="Find a car" desc="Browse hundreds of cars near you. Filter by price, brand, location, and date." />
            <StepCard num="02" icon={<Shield size={28} />} title="Book securely" desc="Pay through Paystack. Every trip comes with a protection plan. Host approves instantly." />
            <StepCard num="03" icon={<ArrowRight size={28} />} title="Pick up & drive" desc="Meet the host, inspect the car, and hit the road. Return when you're done." />
          </div>
        </div>
      </section>

      {/* HOST CTA */}
      <section id="host" className="py-[100px] bg-[#0F0F0F] relative overflow-hidden">
        <div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] rounded-full"
          style={{ background: 'radial-gradient(circle, rgba(34,197,94,0.1) 0%, transparent 70%)' }}
        />

        <div className="max-w-[800px] mx-auto px-8 text-center relative z-[1]">
          <FadeIn>
            <p className="text-[13px] uppercase tracking-[3px] text-accent mb-3">For car owners</p>
            <h2 className="font-black mb-5" style={{ fontSize: 'clamp(32px, 5vw, 56px)', letterSpacing: '-2px' }}>
              Your car could be earning.
            </h2>
            <p className="text-lg text-gray-500 leading-relaxed mb-10">
              The average Qent host earns up to ₦300,000/month. Your car sits idle most of the time — put it to work.
            </p>

            <div className="flex justify-center gap-12 mb-12 flex-wrap">
              <div>
                <div className="text-4xl font-black text-accent">85%</div>
                <div className="text-[13px] text-gray-500 mt-1">You keep per booking</div>
              </div>
              <div>
                <div className="text-4xl font-black">₦0</div>
                <div className="text-[13px] text-gray-500 mt-1">To list your car</div>
              </div>
              <div>
                <div className="text-4xl font-black">24h</div>
                <div className="text-[13px] text-gray-500 mt-1">Average payout</div>
              </div>
            </div>

            <Link
              to="/signup"
              className="inline-flex items-center gap-2 px-9 py-4 bg-white text-black rounded-full text-base font-bold transition-transform hover:scale-105"
            >
              Start earning <ArrowRight size={18} />
            </Link>
          </FadeIn>
        </div>
      </section>

      {/* FEATURES GRID */}
      <section className="py-[100px] bg-black">
        <div className="max-w-[1280px] mx-auto px-8">
          <FadeIn>
            <h2
              className="font-black text-center mb-16"
              style={{ fontSize: 'clamp(28px, 4vw, 48px)', letterSpacing: '-2px' }}
            >
              Built different.
            </h2>
          </FadeIn>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <FeatureCard icon={<Shield size={24} color="var(--accent)" />} title="Verified users" desc="Every renter and host is ID-verified before they can book or list." />
            <FeatureCard icon={<Wallet size={24} color="var(--accent)" />} title="Instant payouts" desc="Hosts withdraw earnings to any Nigerian bank account. No delays." />
            <FeatureCard icon={<Clock size={24} color="var(--accent)" />} title="Book in minutes" desc="No paperwork, no agencies. Find a car and book in under 2 minutes." />
            <FeatureCard icon={<Star size={24} color="var(--accent)" />} title="Ratings & reviews" desc="Real reviews from real trips. Know who you're renting from." />
            <FeatureCard icon={<MapPin size={24} color="var(--accent)" />} title="Cars everywhere" desc="Lagos, Abuja, PH, Kano, and more cities coming soon." />
            <FeatureCard icon={<Search size={24} color="var(--accent)" />} title="Smart matching" desc="Personalized recommendations based on your preferences and history." />
          </div>
        </div>
      </section>

      {/* FOOTER */}
      <footer className="pt-[60px] pb-10 border-t border-white/[0.06]">
        <div className="max-w-[1280px] mx-auto px-8 flex justify-between items-center flex-wrap gap-4">
          <div className="flex items-center gap-2.5">
            <Logo variant="full" size={28} />
          </div>
          <p className="text-[13px] text-gray-500">© 2026 Qent. All rights reserved.</p>
          <div className="flex gap-6">
            <a href="#" className="text-[13px] text-gray-500">Privacy</a>
            <a href="#" className="text-[13px] text-gray-500">Terms</a>
          </div>
        </div>
      </footer>
    </div>
  );
}

function Stat({ value, label }) {
  return (
    <div>
      <div className="text-2xl font-extrabold tracking-tight">{value}</div>
      <div className="text-[13px] text-white/40 mt-0.5">{label}</div>
    </div>
  );
}

function CarSection({ title, subtitle, cars }) {
  return (
    <FadeIn>
      <div className="mb-16">
        <div className="flex justify-between items-end mb-6">
          <div>
            <h2 className="text-[28px] font-extrabold tracking-tight mb-1">{title}</h2>
            {subtitle && <p className="text-gray-500 text-sm">{subtitle}</p>}
          </div>
          <span className="text-sm font-semibold text-accent cursor-pointer flex items-center gap-1">
            View all <ChevronRight size={16} />
          </span>
        </div>
        <div className="grid gap-5" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))' }}>
          {cars.slice(0, 6).map(car => <CarCard key={car.id} car={car} />)}
        </div>
      </div>
    </FadeIn>
  );
}

function StepCard({ num, icon, title, desc }) {
  const [hovered, setHovered] = useState(false);
  return (
    <FadeIn delay={Number(num) * 0.1}>
      <div
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        className={`p-9 rounded-3xl border transition-all duration-300 ${
          hovered ? 'bg-white/[0.06] border-accent/20' : 'bg-white/[0.03] border-white/[0.06]'
        }`}
      >
        <div className="text-5xl font-black text-white/[0.06] mb-5">{num}</div>
        <div className="mb-4 text-accent">{icon}</div>
        <h3 className="text-xl font-bold mb-2">{title}</h3>
        <p className="text-gray-500 text-sm leading-relaxed">{desc}</p>
      </div>
    </FadeIn>
  );
}

function FeatureCard({ icon, title, desc }) {
  const [hovered, setHovered] = useState(false);
  return (
    <FadeIn>
      <div
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        className={`p-8 rounded-[20px] border border-white/[0.06] transition-all duration-300 ${
          hovered ? 'bg-white/[0.06]' : 'bg-white/[0.03]'
        }`}
      >
        <div className="w-12 h-12 rounded-2xl bg-accent/10 flex items-center justify-center mb-5">{icon}</div>
        <h3 className="text-[17px] font-bold mb-2">{title}</h3>
        <p className="text-gray-500 text-sm leading-relaxed">{desc}</p>
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
