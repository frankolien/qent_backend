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
      <motion.nav
        initial={{ y: -20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.5 }}
        className={`fixed top-0 left-0 right-0 z-[100] py-4 transition-all duration-300 ${
          scrolled || menuOpen
            ? 'bg-black/95 backdrop-blur-nav border-b border-white/[0.06]'
            : 'bg-transparent border-b-0'
        }`}
      >
        <div className="max-w-[1280px] mx-auto px-6 flex justify-between items-center">
          <Link to="/" className="flex items-center gap-2.5 no-underline">
            <Logo variant="full" size={36} />
          </Link>

          {/* Center nav links — desktop only */}
          <div className="hidden md:flex items-center gap-8">
            <NavLink href="#cars" active={isHome} onClick={() => handleNavLinkClick('#cars')}>Browse</NavLink>
            <NavLink href="#how-it-works" onClick={() => handleNavLinkClick('#how-it-works')}>How it works</NavLink>
            {user ? (
              <Link to="/dashboard" className="text-sm font-medium text-white/50 cursor-pointer no-underline transition-colors hover:text-white">
                Partnership
              </Link>
            ) : (
              <NavLink href="#host" onClick={() => handleNavLinkClick('#host')}>Become a host</NavLink>
            )}
          </div>

          {/* Right — desktop only */}
          <div className="hidden md:flex items-center gap-3">
            {user ? (
              <>
                <Link to="/trips" className="px-[18px] py-2 text-[13px] font-semibold text-white bg-white/[0.08] border border-white/10 rounded-full no-underline">
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
                <Link to="/login" className="text-sm font-medium text-white/70 px-4 py-2 no-underline">Log in</Link>
                <Link to="/signup" className="px-6 py-2.5 text-sm font-semibold bg-accent text-black rounded-full transition-transform hover:scale-105 no-underline">
                  Get Started
                </Link>
              </>
            )}
          </div>

          {/* Hamburger — mobile only */}
          <button
            onClick={() => setMenuOpen(prev => !prev)}
            aria-label="Toggle menu"
            className="md:hidden flex flex-col justify-center items-center gap-[5px] w-10 h-10 bg-white/[0.08] border border-white/10 rounded-[10px] cursor-pointer p-0 flex-shrink-0"
          >
            <motion.span
              animate={menuOpen ? { rotate: 45, y: 7 } : { rotate: 0, y: 0 }}
              transition={{ duration: 0.25 }}
              className="block w-[18px] h-[1.5px] bg-white rounded-[2px] origin-center"
            />
            <motion.span
              animate={menuOpen ? { opacity: 0, scaleX: 0 } : { opacity: 1, scaleX: 1 }}
              transition={{ duration: 0.2 }}
              className="block w-[18px] h-[1.5px] bg-white rounded-[2px]"
            />
            <motion.span
              animate={menuOpen ? { rotate: -45, y: -7 } : { rotate: 0, y: 0 }}
              transition={{ duration: 0.25 }}
              className="block w-[18px] h-[1.5px] bg-white rounded-[2px] origin-center"
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
            className="fixed top-[70px] left-0 right-0 bottom-0 z-[99] flex flex-col overflow-y-auto px-6 pb-10 pt-4"
            style={{ background: 'rgba(10,10,10,0.98)', backdropFilter: 'blur(24px)', WebkitBackdropFilter: 'blur(24px)' }}
          >
            {/* Nav links */}
            <div className="flex flex-col gap-0.5 mb-8">
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
            <div className="h-px bg-white/[0.07] mb-8" />

            {/* Auth section */}
            {user ? (
              <div className="flex flex-col gap-3">
                <div
                  onClick={() => { setMenuOpen(false); navigate('/profile'); }}
                  className="flex items-center gap-3.5 py-3 cursor-pointer"
                >
                  <div className="w-11 h-11 rounded-xl bg-white/10 border border-white/10 flex items-center justify-center overflow-hidden flex-shrink-0">
                    {user.profile_photo_url ? (
                      <img src={user.profile_photo_url} alt="" className="w-full h-full object-cover" />
                    ) : (
                      <span className="text-base font-bold text-white">{user.full_name?.[0]}</span>
                    )}
                  </div>
                  <div>
                    <div className="text-[15px] font-semibold text-white">{user.full_name}</div>
                    <div className="text-[13px] text-white/40 mt-0.5">View profile</div>
                  </div>
                </div>
                <button
                  onClick={() => { logout(); setMenuOpen(false); }}
                  className="mt-2 py-3.5 text-sm font-semibold bg-white/[0.06] border border-white/[0.08] rounded-xl text-white/60 cursor-pointer w-full"
                >
                  Log out
                </button>
              </div>
            ) : (
              <div className="flex flex-col gap-3">
                <Link
                  to="/signup"
                  onClick={() => setMenuOpen(false)}
                  className="block text-center py-[15px] px-6 text-[15px] font-semibold bg-accent text-black rounded-2xl no-underline"
                >
                  Get Started
                </Link>
                <Link
                  to="/login"
                  onClick={() => setMenuOpen(false)}
                  className="block text-center py-[15px] px-6 text-[15px] font-semibold bg-white/[0.06] border border-white/[0.08] rounded-2xl text-white/70 no-underline"
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

function MobileNavLink({ onClick, children }) {
  return (
    <button
      onClick={onClick}
      className="block text-left w-full py-4 px-1 text-[22px] font-semibold text-white/85 bg-transparent border-0 border-b border-white/[0.05] cursor-pointer tracking-tight"
    >
      {children}
    </button>
  );
}
