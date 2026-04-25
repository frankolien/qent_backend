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
    <div className="max-w-[1280px] mx-auto pt-[100px] px-8 pb-20">
      {/* Search bar */}
      <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.4 }}>
        <form onSubmit={handleSubmit} className="flex gap-3 mb-8">
          <div className="flex-1 flex items-center bg-white/5 border border-white/[0.08] rounded-2xl px-5">
            <Search size={18} color="var(--gray-500)" />
            <input
              value={query} onChange={e => setQuery(e.target.value)}
              placeholder="Search by location, brand, model..."
              className="flex-1 border-0 bg-transparent px-3.5 py-4 text-[15px] text-white outline-none"
            />
            {query && <X size={16} color="var(--gray-500)" className="cursor-pointer" onClick={() => setQuery('')} />}
          </div>
          <button
            type="submit"
            className="px-7 bg-accent border-0 rounded-2xl text-sm font-bold text-black cursor-pointer"
          >
            Search
          </button>
          <button
            type="button"
            onClick={() => setShowFilters(!showFilters)}
            className={`w-[52px] h-[52px] rounded-2xl border border-white/[0.08] flex items-center justify-center cursor-pointer ${
              showFilters ? 'bg-accent text-black' : 'bg-white/5 text-white'
            }`}
          >
            <SlidersHorizontal size={18} />
          </button>
        </form>

        {/* Filters */}
        {showFilters && (
          <motion.div
            initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: 'auto' }}
            className="flex gap-4 mb-8 p-5 rounded-[20px] bg-white/[0.03] border border-white/[0.06] flex-wrap items-end"
          >
            <FilterInput label="Min Price" value={minPrice} onChange={setMinPrice} placeholder="e.g. 10000" />
            <FilterInput label="Max Price" value={maxPrice} onChange={setMaxPrice} placeholder="e.g. 100000" />
            <div>
              <label className="text-xs font-semibold text-gray-400 block mb-1.5">Sort By</label>
              <select
                value={sortBy}
                onChange={e => setSortBy(e.target.value)}
                className="px-4 py-3 rounded-xl border border-white/[0.08] bg-white/[0.04] text-white text-[13px] outline-none"
              >
                {SORT_OPTIONS.map(o => (
                  <option key={o.value} value={o.value} style={{ background: '#1A1A1A' }}>{o.label}</option>
                ))}
              </select>
            </div>
            <button
              onClick={doSearch}
              className="px-6 py-3 bg-accent text-black border-0 rounded-xl text-[13px] font-bold cursor-pointer"
            >
              Apply
            </button>
          </motion.div>
        )}

        {/* Sort pills */}
        <div className="flex gap-2 mb-7 flex-wrap">
          {SORT_OPTIONS.map(o => (
            <button
              key={o.value}
              onClick={() => setSortBy(o.value)}
              className={`px-[18px] py-2 rounded-full border-0 cursor-pointer text-[13px] font-semibold transition-all duration-200 ${
                sortBy === o.value ? 'bg-accent text-black' : 'bg-white/5 text-gray-400'
              }`}
            >
              {o.label}
            </button>
          ))}
        </div>
      </motion.div>

      {/* Results */}
      {loading ? (
        <div className="text-center py-20 text-gray-500">Searching...</div>
      ) : cars.length === 0 ? (
        <div className="text-center py-20">
          <div className="text-5xl mb-4">🚗</div>
          <h2 className="text-[22px] font-bold mb-2">No cars found</h2>
          <p className="text-gray-500 text-sm">Try adjusting your search or filters</p>
        </div>
      ) : (
        <>
          <p className="text-gray-500 text-sm mb-5">{cars.length} car{cars.length === 1 ? '' : 's'} found</p>
          <div className="grid gap-5" style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))' }}>
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
      <label className="text-xs font-semibold text-gray-400 block mb-1.5">{label}</label>
      <input
        value={value}
        onChange={e => onChange(e.target.value)}
        placeholder={placeholder}
        className="px-4 py-3 rounded-xl border border-white/[0.08] bg-white/[0.04] text-white text-[13px] outline-none w-[140px]"
      />
    </div>
  );
}
