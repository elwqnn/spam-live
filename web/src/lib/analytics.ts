export const RANGES = ['24h', '7d', '30d'] as const;
export type Range = (typeof RANGES)[number];

const HOUR = 3_600_000;
const STEP = { '24h': 4, '7d': 24, '30d': 120 };

export function hourTicks(hours: Date[], range: Range, max = Infinity) {
	const step = STEP[range] * Math.max(1, Math.ceil(hours.length / STEP[range] / max));
	return hours.filter((d) => (d.getTime() / HOUR) % step === 0);
}

export function hourLabel(range: Range, locale: string) {
	const format = new Intl.DateTimeFormat(
		locale,
		range === '24h'
			? { hour: 'numeric', timeZone: 'UTC' }
			: { month: 'short', day: 'numeric', timeZone: 'UTC' }
	);
	return (d: Date) => format.format(d);
}
