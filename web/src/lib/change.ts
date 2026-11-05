import { m } from './paraglide/messages.js';
import type { Locale } from './paraglide/runtime.js';

export function formatChange(change: number | null | undefined, locale: Locale) {
	if (change == null) return null;
	const points = Math.abs(change * 100);
	const sign = points.toFixed(1) === '0.0' ? 0 : Math.sign(change);
	const value = new Intl.NumberFormat(locale, {
		minimumFractionDigits: 1,
		maximumFractionDigits: 1
	}).format(points);
	return { sign, value, text: m.points({ value }, { locale }) };
}

const shown = new Intl.NumberFormat('en', { style: 'percent', maximumFractionDigits: 1 });

export function winRateColor(winRate: number | null | undefined) {
	if (winRate == null || shown.format(winRate) === '50%') return undefined;
	const weight = Math.round(Math.min(1, Math.abs(winRate - 0.5) / 0.1) * 100);
	return `color-mix(in oklab, var(${winRate > 0.5 ? '--up' : '--down'}) ${weight}%, var(--foreground))`;
}
