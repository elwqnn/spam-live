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
	const unit = m.points({ value }, { locale });
	return { sign, value, text: sign ? `${sign > 0 ? '▲' : '▼'} ${unit}` : unit };
}
