/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,jsx,ts,tsx}'],
  theme: {
    extend: {
      colors: {
        // Mirror existing CSS variables — keep names so future dev recognizes them
        black: '#0A0A0A',
        white: '#FFFFFF',
        accent: '#22C55E',
        'accent-dark': '#16A34A',
        gray: {
          50: '#FAFAFA',
          100: '#F5F5F5',
          200: '#E5E5E5',
          400: '#A3A3A3',
          500: '#737373',
          600: '#525252',
          800: '#262626',
        },
      },
      fontFamily: {
        sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'sans-serif'],
      },
      backdropBlur: {
        nav: '24px',
        glass: '20px',
      },
    },
  },
  plugins: [],
}
