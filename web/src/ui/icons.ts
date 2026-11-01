// Small drawn icons shared by several screens.

/** A padlock, drawn. */
export function lockIcon(size = 22): string {
  return `<svg class="lock-icon" viewBox="0 0 24 24" width="${size}" height="${size}" aria-hidden="true"><path d="M7 11 V8 a5 5 0 0 1 10 0 V11" fill="none" stroke="currentColor" stroke-width="2.4"/><rect x="4.5" y="10.5" width="15" height="11" rx="2" fill="currentColor"/><circle cx="12" cy="15" r="1.7" fill="#2b1d10"/><path d="M12 16 L12 18.5" stroke="#2b1d10" stroke-width="1.6"/></svg>`;
}

/** Two linked rings, for sharing a link. */
export function linkIcon(size = 18): string {
  return `<svg class="link-icon" viewBox="0 0 24 24" width="${size}" height="${size}" aria-hidden="true"><path d="M10 14 a4 4 0 0 0 5.7 0 l3-3 a4 4 0 0 0 -5.7 -5.7 l-1.2 1.2 M14 10 a4 4 0 0 0 -5.7 0 l-3 3 a4 4 0 0 0 5.7 5.7 l1.2 -1.2" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"/></svg>`;
}

/** A calendar sheet with the day of the month, for the daily seed. */
export function calendarIcon(day: number, size = 40): string {
  return `<svg class="calendar-icon" viewBox="0 0 40 40" width="${size}" height="${size}" aria-hidden="true"><rect x="4" y="7" width="32" height="29" rx="3" fill="#fbf3e2" stroke="#2b1a0e" stroke-width="2"/><path d="M4 10 a3 3 0 0 1 3 -3 h26 a3 3 0 0 1 3 3 v5 h-32 Z" fill="#8e2430" stroke="#2b1a0e" stroke-width="2"/><path d="M12 4 V10 M28 4 V10" stroke="#2b1a0e" stroke-width="2.4" stroke-linecap="round"/><text x="20" y="31.5" text-anchor="middle" font-size="14" font-weight="700" fill="#2b1a0e">${day}</text></svg>`;
}
