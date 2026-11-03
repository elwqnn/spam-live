import { describe, expect, it } from 'vitest';
import { selectImport, type PickedFile } from './import';

const A = '00000000-0000-4000-8000-00000000000a';
const B = '00000000-0000-4000-8000-00000000000b';
const DECKS = '{"userId":"x","added":[],"updated":[],"removed":[]}';
const line = `<color=#FFA000>[UserData]</color> Sending decks to server: ${DECKS}`;

const paths = async (files: PickedFile[]) => {
	const { groups, deckLines } = await selectImport(files);
	return {
		groups: groups.map((g) => [g.accountId, g.index.path, g.replays.map((r) => r.path)]),
		deckLines
	};
};

describe('selectImport', () => {
	it('finds nothing in a wrong folder', async () => {
		const files = [
			{ path: `replays/api.crocodyle.fr/${A}/index.json` },
			{ path: `Ticroco/Spam/replays/api.crocodyle.fr/${A}/index.json` },
			{ path: `Spam/replays/${A}/index.json` },
			{ path: 'Spam/Unity/Player.log', text: line }
		];
		expect(await paths(files)).toEqual({ groups: [], deckLines: [] });
	});

	it('keeps one group per AccountId with its replays', async () => {
		const files = [
			{ path: `Spam/replays/api.crocodyle.fr/${B}/match-2.srpl` },
			{ path: `Spam/replays/api.crocodyle.fr/${B}/index.json` },
			{ path: `Spam/replays/api.crocodyle.fr/${A}/index.json` },
			{ path: `Spam/replays/api.crocodyle.fr/${A}/match-1.srpl` },
			{ path: `Spam/replays/api.crocodyle.fr/${A}/other.srpl` },
			{ path: `Spam/replays/api.crocodyle.fr/${A}/old/match-0.srpl` },
			{ path: 'Spam/replays/api.crocodyle.fr/no-index/match-3.srpl' },
			{ path: 'Spam/usage_counters.json' },
			{ path: 'Spam/Unity/Analytics/values' }
		];
		expect((await paths(files)).groups).toEqual([
			[
				A,
				`Spam/replays/api.crocodyle.fr/${A}/index.json`,
				[`Spam/replays/api.crocodyle.fr/${A}/match-1.srpl`]
			],
			[
				B,
				`Spam/replays/api.crocodyle.fr/${B}/index.json`,
				[`Spam/replays/api.crocodyle.fr/${B}/match-2.srpl`]
			]
		]);
	});

	it('ignores other servers', async () => {
		const files = [
			{ path: `Spam/replays/test.crocodyle.fr/${A}/index.json` },
			{ path: `Spam/replays/test.crocodyle.fr/${A}/match-1.srpl` },
			{ path: `Spam/replays/api.crocodyle.fr.evil/${A}/index.json` }
		];
		expect((await paths(files)).groups).toEqual([]);
	});

	it('keeps only the deck lines, from the marker', async () => {
		const text = ['Initialize engine version', line, 'GfxDevice: creating device client', ''].join(
			'\r\n'
		);
		const files = [{ path: 'Spam/Player.log', text }];
		expect((await paths(files)).deckLines).toEqual([`Sending decks to server: ${DECKS}`]);
	});

	it('reads Player-prev.log before Player.log', async () => {
		const files = [
			{ path: 'Spam/Player.log', file: new File([`${line}current`], 'Player.log') },
			{ path: 'Spam/Player-prev.log', file: new File([`${line}prev`], 'Player-prev.log') },
			{ path: 'Spam/Player.log.bak', text: line }
		];
		expect((await paths(files)).deckLines).toEqual([
			`Sending decks to server: ${DECKS}prev`,
			`Sending decks to server: ${DECKS}current`
		]);
	});

	it('accepts Windows and Linux paths', async () => {
		for (const sep of ['/', '\\']) {
			const files = [
				{ path: ['Spam', 'replays', 'api.crocodyle.fr', A, 'index.json'].join(sep) },
				{ path: ['Spam', 'replays', 'api.crocodyle.fr', A, 'match-1.srpl'].join(sep) },
				{ path: ['Spam', 'Player.log'].join(sep), text: line }
			];
			const { groups, deckLines } = await paths(files);
			expect(groups.map((g) => [g[0], (g[2] as string[]).length])).toEqual([[A, 1]]);
			expect(deckLines).toHaveLength(1);
		}
	});
});
