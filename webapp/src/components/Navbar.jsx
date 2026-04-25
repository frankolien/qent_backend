import { Link, useNavigate, useLocation } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';
import { motion } from 'framer-motion';
import { useState, useEffect } from 'react';
import Logo from './Logo';

export default function Navbar() {
  const { user } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const [scrolled, setScrolled] = useState(false);
  const isHome = location.pathname === '/';

  useEffect(() => {
    const handleScroll = () => setScrolled(window.scrollY > 40);
    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  return (
    <motion.nav
      initial={{ y: -20, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      transition={{ duration: 0.5 }}
      className={`fixed top-0 left-0 right-0 z-[100] py-4 transition-all duration-300 ${
        scrolled
          ? 'bg-black/80 backdrop-blur-nav border-b border-white/[0.06]'
          : 'bg-transparent border-b-0'
      }`}
    >
      <div className="max-w-[1280px] mx-auto px-8 flex justify-between items-center">
        <Link to="/" className="flex items-center gap-2.5 no-underline">
          <Logo variant="full" size={36} />
        </Link>

        {/* Center nav links */}
        <div className="flex items-center gap-8">
          <NavLink href="#cars" active={isHome}>Browse</NavLink>
          <NavLink href="#how-it-works">How it works</NavLink>
          {user ? (
            <Link
              to="/dashboard"
              className="text-sm font-medium text-white/50 cursor-pointer no-underline transition-colors hover:text-white"
            >
              Partnership
            </Link>
          ) : (
            <NavLink href="#host">Become a host</NavLink>
          )}
        </div>

        {/* Right */}
        <div className="flex items-center gap-3">
          {user ? (
            <>
              <Link
                to="/trips"
                className="px-[18px] py-2 text-[13px] font-semibold text-white bg-white/[0.08] border border-white/10 rounded-full"
              >
                My trips
              </Link>
              <div
                onClick={() => navigate('/profile')}
                className="w-[38px] h-[38px] rounded-xl bg-white/10 border border-white/10 flex items-center justify-center cursor-pointer overflow-hidden"
              >
                {user.profile_photo_url ? (
                  <img src={user.profile_photo_url} alt="" className="w-full h-full object-cover" />
                ) : (
                  <span className="text-sm font-bold">{user.full_name?.[0]}</span>
                )}
              </div>
            </>
          ) : (
            <>
              <Link to="/login" className="text-sm font-medium text-white/70 px-4 py-2">Log in</Link>
              <Link
                to="/signup"
                className="px-6 py-2.5 text-sm font-semibold bg-accent text-black rounded-full transition-transform hover:scale-105"
              >
                Get Started
              </Link>
            </>
          )}
        </div>
      </div>
    </motion.nav>
  );
}

function NavLink({ href, active, children }) {
  const navigate = useNavigate();
  const handleClick = (e) => {
    e.preventDefault();
    const el = document.querySelector(href);
    if (el) {
      el.scrollIntoView({ behavior: 'smooth' });
    } else {
      navigate('/');
      setTimeout(() => {
        const target = document.querySelector(href);
        if (target) target.scrollIntoView({ behavior: 'smooth' });
      }, 100);
    }
  };

  return (
    <a
      href={href}
      onClick={handleClick}
      className={`text-sm font-medium cursor-pointer transition-colors hover:text-white ${
        active ? 'text-white' : 'text-white/50'
      }`}
    >
      {children}
    </a>
  );
}
