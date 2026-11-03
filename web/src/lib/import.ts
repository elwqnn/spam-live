export const MARKER = 'Sending decks to server: ';
export const LOGS = ['Player-prev.log', 'Player.log'];

export type PickedFile = { path: string; text?: string; file?: File };
export type AccountFiles = { accountId: string; index: PickedFile; replays: PickedFile[] };

const REPLAY = /^replays\/api\.crocodyle\.fr\/([^/]+)\/(index\.json|match-[^/]+\.srpl)$/;

export async function selectImport(files: PickedFile[]) {
	const accounts = new Map<string, { index?: PickedFile; replays: PickedFile[] }>();
	const logs = new Map<string, PickedFile>();
	for (const f of files) {
		const rel = f.path.replace(/\\/g, '/').split('/').slice(1).join('/');
		if (LOGS.includes(rel)) logs.set(rel, f);
		const match = REPLAY.exec(rel);
		if (!match) continue;
		const account = accounts.get(match[1]) ?? { replays: [] };
		accounts.set(match[1], account);
		if (match[2] === 'index.json') account.index = f;
		else account.replays.push(f);
	}
	const groups: AccountFiles[] = [...accounts]
		.flatMap(([accountId, { index, replays }]) => (index ? [{ accountId, index, replays }] : []))
		.sort((a, b) => a.accountId.localeCompare(b.accountId));
	const deckLines: string[] = [];
	for (const name of LOGS) {
		const log = logs.get(name);
		if (!log) continue;
		const text = log.text ?? (await log.file?.text()) ?? '';
		for (const line of text.split(/\r?\n/)) {
			const at = line.indexOf(MARKER);
			if (at >= 0) deckLines.push(line.slice(at));
		}
	}
	return { groups, deckLines };
}
