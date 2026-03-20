import { Link, useNavigate, useLocation } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';
import { motion, AnimatePresence } from 'framer-motion';
import { useState, useEffect } from 'react';
import Logo from './Logo';

export default function Navbar() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const [scrolled, setScrolled] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const isHome = location.pathname === '/';

  useEffect(() => {
    const handleScroll = () => setScrolled(window.scrollY > 40);
    window.addEventListener('scroll', handleScroll);
    return () => window.removeEventListener('scroll', handleScroll);
  }, []);

  // Lock body scroll when mobile menu is open
  useEffect(() => {
    document.body.style.overflow = menuOpen ? 'hidden' : '';
    return () => { document.body.style.overflow = ''; };
  }, [menuOpen]);

  // Close menu on route change
  useEffect(() => {
    setMenuOpen(false);
  }, [location.pathname]);

  const handleNavLinkClick = (href) => {
    setMenuOpen(false);
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
    <>
      <style>{`
        @media (max-width: 768px) {
          .nav-center-links { display: none !important; }
          .nav-right-desktop { display: none !important; }
          .nav-hamburger { display: flex !important; }
        }
        @media (min-width: 769px) {
          .nav-hamburger { display: none !important; }
        }
      `}</style>

      <motion.nav
        initial={{ y: -20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.5 }}
        style={{
          position: 'fixed', top: 0, left: 0, right: 0, zIndex: 100,
          padding: '16px 0',
          background: (scrolled || menuOpen) ? 'rgba(10,10,10,0.95)' : 'transparent',
          backdropFilter: (scrolled || menuOpen) ? 'blur(24px)' : 'none',
          WebkitBackdropFilter: (scrolled || menuOpen) ? 'blur(24px)' : 'none',
          borderBottom: (scrolled || menuOpen) ? '1px solid rgba(255,255,255,0.06)' : 'none',
          transition: 'all 0.3s ease',
        }}
      >
        <div style={{
          maxWidth: 1280, margin: '0 auto',
          padding: '0 24px',
          display: 'flex', justifyContent: 'space-between', alignItems: 'center',
        }}>
          {/* Logo */}
          <Link to="/" style={{ display: 'flex', alignItems: 'center', gap: 10, textDecoration: 'none' }}>
            <Logo variant="full" size={36} />
          </Link>

          {/* Center nav links — desktop only */}
          <div className="nav-center-links" style={{ display: 'flex', gap: 32, alignItems: 'center' }}>
            <NavLink href="#cars" active={isHome} onClick={() => handleNavLinkClick('#cars')}>Browse</NavLink>
            <NavLink href="#how-it-works" onClick={() => handleNavLinkClick('#how-it-works')}>How it works</NavLink>
            {user ? (
              <Link to="/dashboard" style={navLinkStyle}>Partnership</Link>
            ) : (
              <NavLink href="#host" onClick={() => handleNavLinkClick('#host')}>Become a host</NavLink>
            )}
          </div>

          {/* Right — desktop only */}
          <div className="nav-right-desktop" style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
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
                <Link to="/login" style={{ fontSize: 14, fontWeight: 500, color: 'rgba(255,255,255,0.7)', padding: '8px 16px', textDecoration: 'none' }}>Log in</Link>
                <Link to="/signup" style={{
                  padding: '10px 24px', fontSize: 14, fontWeight: 600,
                  background: 'var(--accent)', color: 'var(--black)',
                  borderRadius: 100, transition: 'transform 0.2s', textDecoration: 'none',
                }}>Get Started</Link>
              </>
            )}
          </div>

          {/* Hamburger — mobile only */}
          <button
            className="nav-hamburger"
            onClick={() => setMenuOpen(prev => !prev)}
            aria-label="Toggle menu"
            style={{
              display: 'flex',
              flexDirection: 'column', justifyContent: 'center', alignItems: 'center',
              gap: 5, width: 40, height: 40,
              background: 'rgba(255,255,255,0.08)',
              border: '1px solid rgba(255,255,255,0.1)',
              borderRadius: 10, cursor: 'pointer', padding: 0,
              flexShrink: 0,
            }}
          >
            <motion.span
              animate={menuOpen ? { rotate: 45, y: 7 } : { rotate: 0, y: 0 }}
              transition={{ duration: 0.25 }}
              style={{ display: 'block', width: 18, height: 1.5, background: 'white', borderRadius: 2, transformOrigin: 'center' }}
            />
            <motion.span
              animate={menuOpen ? { opacity: 0, scaleX: 0 } : { opacity: 1, scaleX: 1 }}
              transition={{ duration: 0.2 }}
              style={{ display: 'block', width: 18, height: 1.5, background: 'white', borderRadius: 2 }}
            />
            <motion.span
              animate={menuOpen ? { rotate: -45, y: -7 } : { rotate: 0, y: 0 }}
              transition={{ duration: 0.25 }}
              style={{ display: 'block', width: 18, height: 1.5, background: 'white', borderRadius: 2, transformOrigin: 'center' }}
            />
          </button>
        </div>
      </motion.nav>

      {/* Mobile Menu Overlay */}
      <AnimatePresence>
        {menuOpen && (
          <motion.div
            key="mobile-menu"
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={{ duration: 0.25, ease: 'easeOut' }}
            style={{
              position: 'fixed', top: 70, left: 0, right: 0, bottom: 0,
              zIndex: 99,
              background: 'rgba(10,10,10,0.98)',
              backdropFilter: 'blur(24px)',
              WebkitBackdropFilter: 'blur(24px)',
              display: 'flex', flexDirection: 'column',
              padding: '16px 24px 40px',
              overflowY: 'auto',
            }}
          >
            {/* Nav links */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: 2, marginBottom: 32 }}>
              <MobileNavLink onClick={() => handleNavLinkClick('#cars')}>Browse</MobileNavLink>
              <MobileNavLink onClick={() => handleNavLinkClick('#how-it-works')}>How it works</MobileNavLink>
              {user ? (
                <MobileNavLink onClick={() => { setMenuOpen(false); navigate('/dashboard'); }}>Partnership</MobileNavLink>
              ) : (
                <MobileNavLink onClick={() => handleNavLinkClick('#host')}>Become a host</MobileNavLink>
              )}
              {user && (
                <MobileNavLink onClick={() => { setMenuOpen(false); navigate('/trips'); }}>My trips</MobileNavLink>
              )}
            </div>

            {/* Divider */}
            <div style={{ height: 1, background: 'rgba(255,255,255,0.07)', marginBottom: 32 }} />

            {/* Auth section */}
            {user ? (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <div
                  onClick={() => { setMenuOpen(false); navigate('/profile'); }}
                  style={{
                    display: 'flex', alignItems: 'center', gap: 14,
                    padding: '12px 0', cursor: 'pointer',
                  }}
                >
                  <div style={{
                    width: 44, height: 44, borderRadius: 12,
                    background: 'rgba(255,255,255,0.1)', border: '1px solid rgba(255,255,255,0.1)',
                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                    overflow: 'hidden', flexShrink: 0,
                  }}>
                    {user.profile_photo_url ? (
                      <img src={user.profile_photo_url} alt="" style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
                    ) : (
                      <span style={{ fontSize: 16, fontWeight: 700, color: 'white' }}>{user.full_name?.[0]}</span>
                    )}
                  </div>
                  <div>
                    <div style={{ fontSize: 15, fontWeight: 600, color: 'white' }}>{user.full_name}</div>
                    <div style={{ fontSize: 13, color: 'rgba(255,255,255,0.4)', marginTop: 1 }}>View profile</div>
                  </div>
                </div>
                <button
                  onClick={() => { logout(); setMenuOpen(false); }}
                  style={{
                    marginTop: 8, padding: '14px', fontSize: 14, fontWeight: 600,
                    background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
                    borderRadius: 12, color: 'rgba(255,255,255,0.6)', cursor: 'pointer',
                    width: '100%',
                  }}
                >
                  Log out
                </button>
              </div>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <Link
                  to="/signup"
                  onClick={() => setMenuOpen(false)}
                  style={{
                    display: 'block', textAlign: 'center',
                    padding: '15px 24px', fontSize: 15, fontWeight: 600,
                    background: 'var(--accent)', color: 'var(--black)',
                    borderRadius: 14, textDecoration: 'none',
                  }}
                >
                  Get Started
                </Link>
                <Link
                  to="/login"
                  onClick={() => setMenuOpen(false)}
                  style={{
                    display: 'block', textAlign: 'center',
                    padding: '15px 24px', fontSize: 15, fontWeight: 600,
                    background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.08)',
                    borderRadius: 14, color: 'rgba(255,255,255,0.7)', textDecoration: 'none',
                  }}
                >
                  Log in
                </Link>
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}

function NavLink({ href, active, onClick, children }) {
  const navigate = useNavigate();
  const handleClick = (e) => {
    e.preventDefault();
    if (onClick) { onClick(); return; }
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
    <a href={href} onClick={handleClick} style={{
      fontSize: 14, fontWeight: 500,
      color: active ? 'white' : 'rgba(255,255,255,0.5)',
      transition: 'color 0.2s', cursor: 'pointer', textDecoration: 'none',
    }}>{children}</a>
  );
}

function MobileNavLink({ onClick, children }) {
  return (
    <button
      onClick={onClick}
      style={{
        display: 'block', textAlign: 'left', width: '100%',
        padding: '16px 4px', fontSize: 22, fontWeight: 600,
        color: 'rgba(255,255,255,0.85)', background: 'none', border: 'none',
        cursor: 'pointer', letterSpacing: '-0.3px',
        borderBottom: '1px solid rgba(255,255,255,0.05)',
      }}
    >
      {children}
    </button>
  );
}

const pillStyle = {
  padding: '8px 18px', fontSize: 13, fontWeight: 600,
  background: 'rgba(255,255,255,0.08)', border: '1px solid rgba(255,255,255,0.1)',
  borderRadius: 100, color: 'white', textDecoration: 'none',
};

const navLinkStyle = {
  fontSize: 14, fontWeight: 500,
  color: 'rgba(255,255,255,0.5)',
  transition: 'color 0.2s', cursor: 'pointer', textDecoration: 'none',
};