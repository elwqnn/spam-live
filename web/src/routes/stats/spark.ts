export type Range = [number, number];

const DAY = 864e5;

export function spanRange(values: (number | null)[], minSpan: number): Range {
	const known = values.filter((v) => v != null);
	if (!known.length) return [0.5 - minSpan / 2, 0.5 + minSpan / 2];
	const lo = Math.min(...known);
	const hi = Math.max(...known);
	if (hi - lo >= minSpan) return [lo, hi];
	const mid = (lo + hi) / 2;
	return [mid - minSpan / 2, mid + minSpan / 2];
}

export function sparkPath(
	values: (number | null)[],
	[lo, hi]: Range,
	width: number,
	height: number
) {
	const step = values.length > 1 ? width / (values.length - 1) : 0;
	const y = (v: number) => height - ((Math.min(hi, Math.max(lo, v)) - lo) / (hi - lo)) * height;
	const parts: string[] = [];
	let open = false;
	values.forEach((v, i) => {
		if (v == null) {
			open = false;
			return;
		}
		const x = values.length > 1 ? i * step : width / 2;
		const alone = !open && values[i + 1] == null;
		parts.push(`${open ? 'L' : 'M'}${round(x)} ${round(y(v))}${alone ? 'h0' : ''}`);
		open = true;
	});
	return parts.join('');
}

const round = (n: number) => Math.round(n * 100) / 100;

export function curveRates(
	curve: { start: number; games: number; wins: number }[],
	minGames: number
): (number | null)[] {
	if (!curve.length) return [];
	const first = Math.min(...curve.map((g) => g.start));
	const last = Math.max(...curve.map((g) => g.start));
	return Array.from({ length: (last - first) / 3 + 1 }, (_, i) => {
		const group = curve.find((g) => g.start === first + i * 3);
		return group && group.games >= minGames ? group.wins / group.games : null;
	});
}

export const isoDay = (date: Date) => date.toISOString().slice(0, 10);

export function monday(date: Date) {
	const day = Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate());
	return new Date(day - ((new Date(day).getUTCDay() + 6) % 7) * DAY);
}

export function fullWeeks(weeks: string[], now: Date, count: number) {
	const current = isoDay(monday(now));
	const full = [...new Set(weeks)].filter((w) => w < current).sort();
	if (full.length < 2) return null;
	const last = full[full.length - 1];
	const end = new Date(`${last}T00:00:00Z`).getTime();
	const first = new Date(`${full[0]}T00:00:00Z`).getTime();
	const length = Math.min(count, Math.round((end - first) / (7 * DAY)) + 1);
	const recent = Array.from({ length }, (_, i) =>
		isoDay(new Date(end - (length - 1 - i) * 7 * DAY))
	);
	return { last, before: recent[length - 2], recent };
}
