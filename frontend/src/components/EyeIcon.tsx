export default function EyeIcon({ color = "#141414" }: { color?: string }) {
  return (
    <svg viewBox="0 0 24 15" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path
        d="M12 0C6.5 0 1.7 3.3 0 7.5 1.7 11.7 6.5 15 12 15s10.3-3.3 12-7.5C22.3 3.3 17.5 0 12 0z"
        stroke={color}
        strokeWidth="1.4"
      />
      <circle cx="12" cy="7.5" r="3.2" fill={color} />
    </svg>
  );
}
