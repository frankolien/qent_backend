import axios from 'axios';

const API_BASE = import.meta.env.VITE_API_URL || 'http://127.0.0.1:8080/api';

const api = axios.create({
  baseURL: API_BASE,
  headers: { 'Content-Type': 'application/json' },
});

// Attach token to every request
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) config.headers.Authorization = `Bearer ${token}`;
  return config;
});

// Handle 401 → logout
api.interceptors.response.use(
  (res) => res,
  (err) => {
    if (err.response?.status === 401) {
      localStorage.removeItem('token');
      localStorage.removeItem('user');
      window.location.href = '/login';
    }
    return Promise.reject(err);
  }
);

export default api;

// Auth
export const signIn = (email, password) => api.post('/auth/signin', { email, password });
export const signUp = (data) => api.post('/auth/signup', data);
export const getProfile = () => api.get('/auth/profile');

// Cars
export const searchCars = (params) => api.get('/cars/search', { params });
export const getHomepage = (params) => api.get('/cars/homepage', { params });
export const getCar = (id) => api.get(`/cars/${id}`);
export const getBookedDates = (id) => api.get(`/cars/${id}/booked-dates`);

// Bookings
export const createBooking = (data) => api.post('/bookings', data);
export const getMyBookings = () => api.get('/bookings/mine');
export const bookingAction = (id, action, reason) => api.post(`/bookings/${id}/action`, { action, reason });

// Reviews
export const getUserReviews = (id) => api.get(`/users/${id}/reviews`);
export const getUserRating = (id) => api.get(`/users/${id}/rating`);
export const createReview = (data) => api.post('/reviews', data);

// Favorites
export const getFavorites = () => api.get('/favorites');
export const toggleFavorite = (id) => api.post(`/favorites/${id}`);
export const checkFavorite = (id) => api.get(`/favorites/${id}/check`);

// Wallet
export const getWallet = () => api.get('/payments/wallet');
export const getEarnings = () => api.get('/payments/earnings');
export const getTransactions = () => api.get('/payments/wallet/transactions');

// Chat
export const getConversations = () => api.get('/chat/conversations');
export const getMessages = (id) => api.get(`/chat/conversations/${id}/messages`);
export const sendMessage = (id, content) => api.post(`/chat/conversations/${id}/messages`, { content, message_type: 'text' });
