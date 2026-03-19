import { Link, useNavigate, useLocation } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';
import { motion } from 'framer-motion';
import { useState, useEffect } from 'react';
import Logo from './Logo';

export default function Navbar() {
  const { user, logout } = useAuth();
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
      style={{
        position: 'fixed', top: 0, left: 0, right: 0, zIndex: 100,
        padding: '16px 0',
        background: scrolled ? 'rgba(10,10,10,0.8)' : 'transparent',
        backdropFilter: scrolled ? 'blur(24px)' : 'none',
        WebkitBackdropFilter: scrolled ? 'blur(24px)' : 'none',
        borderBottom: scrolled ? '1px solid rgba(255,255,255,0.06)' : 'none',
        transition: 'all 0.3s ease',
      }}
    >
      <div style={{ maxWidth: 1280, margin: '0 auto', padding: '0 32px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Link to="/" style={{ display: 'flex', alignItems: 'center', gap: 10, textDecoration: 'none' }}>
          <Logo variant="full" size={36} />
        </Link>

        {/* Center nav links */}
        <div style={{ display: 'flex', gap: 32, alignItems: 'center' }}>
          <NavLink href="#cars" active={isHome}>Browse</NavLink>
          <NavLink href="#how-it-works">How it works</NavLink>
          {user ? (
            <Link to="/dashboard" style={{ fontSize: 14, fontWeight: 500, color: 'rgba(255,255,255,0.5)', transition: 'color 0.2s', cursor: 'pointer', textDecoration: 'none' }}>
              Partnership
            </Link>
          ) : (
            <NavLink href="#host">Become a host</NavLink>
          )}
        </div>

        {/* Right */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          {user ? (
            <>
              <Link to="/trips" style={pillStyle}>My trips</Link>
              <div
                onClick={() => navigate('/profile')}
                style={{
                  width: 38, height: 38, borderRadius: 12,
                  background: 'rgba(255,255,255,0.1)', border: '1px solid rgba(255,255,255,0.1)',
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  cursor: 'pointer', overflow: 'hidden',
                }}
              >
                {user.profile_photo_url ? (
                  <img src={user.profile_photo_url} alt="" style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
                ) : (
                  <span style={{ fontSize: 14, fontWeight: 700 }}>{user.full_name?.[0]}</span>
                )}
              </div>
            </>
          ) : (
            <>
              <Link to="/login" style={{ fontSize: 14, fontWeight: 500, color: 'rgba(255,255,255,0.7)', padding: '8px 16px' }}>Log in</Link>
              <Link to="/signup" style={{
                padding: '10px 24px', fontSize: 14, fontWeight: 600,
                background: 'var(--accent)', color: 'var(--black)',
                borderRadius: 100, transition: 'transform 0.2s',
              }}>Get Started</Link>
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
      // Use React Router to navigate without full page reload
      navigate('/');
      setTimeout(() => {
        const target = document.querySelector(href);
        if (target) target.scrollIntoView({ behavior: 'smooth' });
      }, 100);
    }
  };

  return (
    <a href={href} onClick={handleClick} style={{
      fontSize: 14, fontWeight: 500,
      color: active ? 'white' : 'rgba(255,255,255,0.5)',
      transition: 'color 0.2s', cursor: 'pointer',
    }}>{children}</a>
  );
}

const pillStyle = {
  padding: '8px 18px', fontSize: 13, fontWeight: 600,
  background: 'rgba(255,255,255,0.08)', border: '1px solid rgba(255,255,255,0.1)',
  borderRadius: 100, color: 'white',
};
