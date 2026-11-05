export type Reason = 'votes' | 'winrate';

const TOP = 3;
const MIN_VOTES = 5;
const MIN_SCORE = 0.5;

export function promote(
	decks: { winScore?: number | null; votes?: number }[]
): { reasons: Reason[]; large: boolean }[] {
	return decks.map((deck, i) => {
		const reasons: Reason[] = [];
		if (i < TOP && (deck.votes ?? 0) >= MIN_VOTES) reasons.push('votes');
		if (i < TOP && (deck.winScore ?? 0) >= MIN_SCORE) reasons.push('winrate');
		return { reasons, large: i === 0 && reasons.length > 0 && decks.length >= TOP };
	});
}
