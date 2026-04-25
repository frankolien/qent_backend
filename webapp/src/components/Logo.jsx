/**
 * Qent Logo — Stylized Q with a road/speed element
 * Use: <Logo size={36} /> or <Logo size={36} variant="full" />
 *
 * Note: `size` drives pixel dimensions, so size-derived values stay as inline
 * style. Static layout/typography moved to Tailwind classes.
 */
export default function Logo({ size = 36, color = 'currentColor', bg = 'var(--accent)', variant = 'icon' }) {
  if (variant === 'full') {
    return (
      <div className="flex items-center gap-2.5">
        <LogoIcon size={size} color={color} bg={bg} />
        <span
          className="font-extrabold tracking-tight text-white"
          style={{
            fontSize: size * 0.6,
            color: color === 'currentColor' ? undefined : color,
          }}
        >
          Qent
        </span>
      </div>
    );
  }
  return <LogoIcon size={size} color={color} bg={bg} />;
}

function LogoIcon({ size, bg }) {
  const isAccentBg = bg === 'var(--accent)' || bg === '#22C55E';
  return (
    <div
      className="relative flex items-center justify-center overflow-hidden"
      style={{
        width: size,
        height: size,
        borderRadius: size * 0.3,
        background: bg,
      }}
    >
      <svg width={size * 0.6} height={size * 0.6} viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path
          d="M20 4C11.16 4 4 11.16 4 20C4 28.84 11.16 36 20 36C23.72 36 27.12 34.72 29.8 32.56L34 36.76L37.76 33L33.56 28.8C35.16 26.4 36 23.32 36 20C36 11.16 28.84 4 20 4ZM20 30C14.48 30 10 25.52 10 20C10 14.48 14.48 10 20 10C25.52 10 30 14.48 30 20C30 22.2 29.28 24.2 28.08 25.84L24 21.76L20.24 25.52L24.84 30.12C23.32 30.68 21.72 31 20 31V30Z"
          fill={isAccentBg ? '#0A0A0A' : 'white'}
        />
        <rect x="2" y="17" width="6" height="2" rx="1" fill={isAccentBg ? 'rgba(0,0,0,0.3)' : 'rgba(255,255,255,0.3)'} />
        <rect x="0" y="21" width="5" height="2" rx="1" fill={isAccentBg ? 'rgba(0,0,0,0.2)' : 'rgba(255,255,255,0.2)'} />
      </svg>
    </div>
  );
}
