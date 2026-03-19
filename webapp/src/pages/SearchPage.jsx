import { useState, useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';
import { motion } from 'framer-motion';
import { Search, SlidersHorizontal, X } from 'lucide-react';
import CarCard from '../components/CarCard';
import { searchCars } from '../utils/api';

const SORT_OPTIONS = [
  { value: '', label: 'Relevance' },
  { value: 'price_asc', label: 'Price: Low to High' },
  { value: 'price_desc', label: 'Price: High to Low' },
  { value: 'rating', label: 'Top Rated' },
  { value: 'newest', label: 'Newest' },
];

export default function SearchPage() {
  const [params] = useSearchParams();
  const [query, setQuery] = useState(params.get('q') || '');
  const [cars, setCars] = useState([]);
  const [loading, setLoading] = useState(true);
  const [sortBy, setSortBy] = useState('');
  const [minPrice, setMinPrice] = useState('');
  const [maxPrice, setMaxPrice] = useState('');
  const [showFilters, setShowFilters] = useState(false);

  const doSearch = async () => {
    setLoading(true);
    try {
      const p = {};
      if (query.trim()) p.location = query;
      if (sortBy) p.sort_by = sortBy;
      if (minPrice) p.min_price = minPrice;
      if (maxPrice) p.max_price = maxPrice;
      p.per_page = 50;
      const res = await searchCars(p);
      setCars(res.data);
    } catch (_) {}
    setLoading(false);
  };

  useEffect(() => { doSearch(); }, [sortBy]);
  useEffect(() => { if (params.get('q')) { setQuery(params.get('q')); doSearch(); } }, [params.get('q')]);

  const handleSubmit = (e) => { e.preventDefault(); doSearch(); };

  return (
    <div style={{ maxWidth: 1280, margin: '0 auto', padding: '100px 32px 80px' }}>
      {/* Search bar */}
      <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.4 }}>
        <form onSubmit={handleSubmit} style={{ display: 'flex', gap: 12, marginBottom: 32 }}>
          <div style={{
            flex: 1, display: 'flex', alignItems: 'center',
            background: 'rgba(255,255,255,0.05)', border: '1px solid rgba(255,255,255,0.08)',
            borderRadius: 16, padding: '0 20px',
          }}>
            <Search size={18} color="var(--gray-500)" />
            <input
              value={query} onChange={e => setQuery(e.target.value)}
              placeholder="Search by location, brand, model..."
              style={{ flex: 1, border: 'none', background: 'none', padding: '16px 14px', fontSize: 15, color: 'white', outline: 'none', fontFamily: 'inherit' }}
            />
            {query && <X size={16} color="var(--gray-500)" style={{ cursor: 'pointer' }} onClick={() => { setQuery(''); }} />}
          </div>
          <button type="submit" style={{
            padding: '0 28px', background: 'var(--accent)', border: 'none',
            borderRadius: 16, fontSize: 14, fontWeight: 700, color: 'var(--black)', fontFamily: 'inherit', cursor: 'pointer',
          }}>Search</button>
          <button type="button" onClick={() => setShowFilters(!showFilters)} style={{
            width: 52, height: 52, borderRadius: 16,
            background: showFilters ? 'var(--accent)' : 'rgba(255,255,255,0.05)',
            border: '1px solid rgba(255,255,255,0.08)',
            color: showFilters ? 'var(--black)' : 'white',
            display: 'flex', alignItems: 'center', justifyContent: 'center', cursor: 'pointer',
          }}>
            <SlidersHorizontal size={18} />
          </button>
        </form>

        {/* Filters */}
        {showFilters && (
          <motion.div
            initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: 'auto' }}
            style={{
              display: 'flex', gap: 16, marginBottom: 32, padding: 20, borderRadius: 20,
              background: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.06)',
              flexWrap: 'wrap', alignItems: 'flex-end',
            }}
          >
            <FilterInput label="Min Price" value={minPrice} onChange={setMinPrice} placeholder="e.g. 10000" />
            <FilterInput label="Max Price" value={maxPrice} onChange={setMaxPrice} placeholder="e.g. 100000" />
            <div>
              <label style={{ fontSize: 12, fontWeight: 600, color: 'var(--gray-400)', display: 'block', marginBottom: 6 }}>Sort By</label>
              <select value={sortBy} onChange={e => setSortBy(e.target.value)} style={{
                padding: '12px 16px', borderRadius: 12, border: '1px solid rgba(255,255,255,0.08)',
                background: 'rgba(255,255,255,0.04)', color: 'white', fontSize: 13, fontFamily: 'inherit', outline: 'none',
              }}>
                {SORT_OPTIONS.map(o => <option key={o.value} value={o.value} style={{ background: '#1A1A1A' }}>{o.label}</option>)}
              </select>
            </div>
            <button onClick={doSearch} style={{
              padding: '12px 24px', background: 'var(--accent)', color: 'var(--black)',
              border: 'none', borderRadius: 12, fontSize: 13, fontWeight: 700, cursor: 'pointer', fontFamily: 'inherit',
            }}>Apply</button>
          </motion.div>
        )}

        {/* Sort pills */}
        <div style={{ display: 'flex', gap: 8, marginBottom: 28, flexWrap: 'wrap' }}>
          {SORT_OPTIONS.map(o => (
            <button key={o.value} onClick={() => setSortBy(o.value)} style={{
              padding: '8px 18px', borderRadius: 100, border: 'none', cursor: 'pointer', fontFamily: 'inherit',
              background: sortBy === o.value ? 'var(--accent)' : 'rgba(255,255,255,0.05)',
              color: sortBy === o.value ? 'var(--black)' : 'var(--gray-400)',
              fontSize: 13, fontWeight: 600, transition: 'all 0.2s',
            }}>{o.label}</button>
          ))}
        </div>
      </motion.div>

      {/* Results */}
      {loading ? (
        <div style={{ textAlign: 'center', padding: 80, color: 'var(--gray-500)' }}>Searching...</div>
      ) : cars.length === 0 ? (
        <div style={{ textAlign: 'center', padding: 80 }}>
          <div style={{ fontSize: 48, marginBottom: 16 }}>&#128663;</div>
          <h2 style={{ fontSize: 22, fontWeight: 700, marginBottom: 8 }}>No cars found</h2>
          <p style={{ color: 'var(--gray-500)', fontSize: 14 }}>Try adjusting your search or filters</p>
        </div>
      ) : (
        <>
          <p style={{ color: 'var(--gray-500)', fontSize: 14, marginBottom: 20 }}>{cars.length} car{cars.length === 1 ? '' : 's'} found</p>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: 20 }}>
            {cars.map(car => <CarCard key={car.id} car={car} />)}
          </div>
        </>
      )}
    </div>
  );
}

function FilterInput({ label, value, onChange, placeholder }) {
  return (
    <div>
      <label style={{ fontSize: 12, fontWeight: 600, color: 'var(--gray-400)', display: 'block', marginBottom: 6 }}>{label}</label>
      <input value={value} onChange={e => onChange(e.target.value)} placeholder={placeholder} style={{
        padding: '12px 16px', borderRadius: 12, border: '1px solid rgba(255,255,255,0.08)',
        background: 'rgba(255,255,255,0.04)', color: 'white', fontSize: 13, fontFamily: 'inherit', outline: 'none', width: 140,
      }} />
    </div>
  );
}
