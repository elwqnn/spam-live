import { m } from '$paraglide/messages.js';

export const STAR_BRACKETS = [0, 15, 30, 45, 60, 75, 90];

export const minStars = (param: string | null) => STAR_BRACKETS.find((b) => String(b) === param);

export const starLabel = (stars: number, ranks: { threshold: number; name: string }[] = []) => {
	const rank = ranks.find((r) => r.threshold === stars)?.name;
	return rank ? m.stars_rank({ rank }) : m.stars_min({ stars });
};
