// The Sift brand mark — a funnel that filters jobs, with the centered dot as
// the right job that made it through. Master vector lives in brand/sift-icon.svg.
export default function BrandMark({ size = 16 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 512 512" aria-hidden>
      <defs>
        <linearGradient id="sift-funnel" x1="170" y1="160" x2="342" y2="362" gradientUnits="userSpaceOnUse">
          <stop stopColor="#6f67f7" />
          <stop offset="1" stopColor="#22d3ee" />
        </linearGradient>
      </defs>
      <rect width="512" height="512" rx="116" fill="#0f141a" />
      <path d="M170 160h172l-66.5 79.4v77.2l-39 23.6V239.4z" fill="url(#sift-funnel)" />
      <circle cx="256" cy="256" r="17" fill="#0f141a" />
    </svg>
  );
}
