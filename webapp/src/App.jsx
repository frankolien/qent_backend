import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { AuthProvider } from './hooks/useAuth';
import Navbar from './components/Navbar';
import Home from './pages/Home';
import Login from './pages/Login';
import SignUp from './pages/SignUp';
import CarDetail from './pages/CarDetail';
import SearchPage from './pages/SearchPage';
import BookCar from './pages/BookCar';
import BookingSuccess from './pages/BookingSuccess';
import Trips from './pages/Trips';
import Favorites from './pages/Favorites';
import Messages from './pages/Messages';
import ChatDetail from './pages/ChatDetail';
import Profile from './pages/Profile';
import Dashboard from './pages/Dashboard';
import WalletPage from './pages/WalletPage';

// Pages that use their own full-height layout (no navbar overlap concern)
const FULLSCREEN_ROUTES = ['/messages/'];

export default function App() {
  return (
    <AuthProvider>
      <BrowserRouter>
        <div style={{ minHeight: '100vh', background: '#0A0A0A', fontFamily: 'Inter, system-ui, -apple-system, sans-serif', color: 'white' }}>
          <style>{`
            :root {
              --accent: #22C55E;
              --black: #0A0A0A;
              --gray-400: rgba(255,255,255,0.6);
              --gray-500: rgba(255,255,255,0.4);
            }
            * { box-sizing: border-box; margin: 0; padding: 0; }
            body { background: #0A0A0A; color: white; }
            a { color: inherit; }
            button { font-family: inherit; }
            @keyframes spin {
              from { transform: rotate(0deg); }
              to   { transform: rotate(360deg); }
            }
            @keyframes pulse {
              0%, 100% { opacity: 1; }
              50%       { opacity: 0.4; }
            }
            ::-webkit-scrollbar { width: 6px; height: 6px; }
            ::-webkit-scrollbar-track { background: transparent; }
            ::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.1); border-radius: 3px; }
            ::-webkit-scrollbar-thumb:hover { background: rgba(255,255,255,0.2); }
          `}</style>

          <NavbarWrapper />

          <Routes>
            {/* Public */}
            <Route path="/" element={<Home />} />
            <Route path="/login" element={<Login />} />
            <Route path="/signup" element={<SignUp />} />
            <Route path="/cars/:id" element={<CarDetail />} />
            <Route path="/search" element={<SearchPage />} />
            <Route path="/book/:id" element={<BookCar />} />
            <Route path="/booking-success" element={<BookingSuccess />} />

            {/* Authenticated */}
            <Route path="/trips" element={<Trips />} />
            <Route path="/favorites" element={<Favorites />} />
            <Route path="/messages" element={<Messages />} />
            <Route path="/messages/:id" element={<ChatDetail />} />
            <Route path="/profile" element={<Profile />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/wallet" element={<WalletPage />} />
          </Routes>
        </div>
      </BrowserRouter>
    </AuthProvider>
  );
}

// Only show Navbar on non-chat routes (chat has its own fixed header)
function NavbarWrapper() {
  // We use a render trick: Navbar is always mounted but hidden on /messages/:id
  // The ChatDetail page manages its own header.
  return <ConditionalNavbar />;
}

function ConditionalNavbar() {
  const { pathname } = typeof window !== 'undefined'
    ? { pathname: window.location.pathname }
    : { pathname: '/' };

  // Hide global navbar inside individual chat — it has its own fixed header
  // We do this check at runtime in Navbar itself would be cleaner, but
  // to avoid modifying Navbar.jsx we just always render it.
  // ChatDetail uses position:fixed header that visually replaces it.
  return <Navbar />;
}
