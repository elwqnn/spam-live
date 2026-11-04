export const ELEMENT_KEYS = {
	FEU: 'fire',
	EAU: 'water',
	TERRE: 'earth',
	NATURE: 'nature',
	LUMIERE: 'light',
	OMBRE: 'shadow'
} as const;

export type Element = (typeof ELEMENT_KEYS)[keyof typeof ELEMENT_KEYS];

export type Segment =
	| { type: 'text'; text: string }
	| { type: 'br' }
	| { type: 'element'; element: Element; label: string }
	| { type: 'keyword'; text: string }
	| { type: 'value'; text: string; dynamic: boolean }
	| { type: 'missing'; name: string };

type Data = {
	actions?: { value?: { x?: unknown; y?: unknown } | null }[];
	duration?: unknown;
	chanceOfResetCooldown?: unknown;
	numberOfActivation?: unknown;
	valueOfEffect?: unknown;
};

const TOKEN = /\n|<(\w+)>|(\p{Lu}+)\[(FEU|EAU|TERRE|NATURE|LUMIERE|OMBRE)\]|\[/gu;
const PLACEHOLDER = /^(value|min|max|limitedUse)(\d+)(D?)$|^(duration|chanceOfReset)(D?)$/;

const num = (v: unknown) => (typeof v === 'number' && Number.isFinite(v) ? v : undefined);

function lookup(data: Data, kind: string, index: number): number | undefined {
	if (kind === 'duration') return num(data.duration);
	if (kind === 'chanceOfReset') return num(data.chanceOfResetCooldown);
	if (kind === 'limitedUse') return num(data.numberOfActivation);
	const value = data.actions?.[index - 1]?.value;
	if (!value) return kind === 'value' && index === 1 ? num(data.valueOfEffect) : undefined;
	const x = num(value.x);
	const y = num(value.y);
	if (kind === 'min') return x;
	if (kind === 'max') return y;
	return x === undefined || y === undefined ? undefined : Math.trunc((x + y) / 2);
}

function placeholder(name: string, data: Data): Segment {
	const m = PLACEHOLDER.exec(name);
	const value = m ? lookup(data, m[1] ?? m[4], Number(m[2] ?? 0)) : undefined;
	if (!m || value === undefined) return { type: 'missing', name };
	return { type: 'value', text: String(value), dynamic: (m[3] ?? m[5]) === 'D' };
}

function closing(text: string, start: number) {
	let depth = 0;
	for (let i = start; i < text.length; i++) {
		if (text[i] === '[') depth++;
		else if (text[i] === ']' && --depth === 0) return i;
	}
	return -1;
}

function pushText(out: Segment[], text: string) {
	if (!text) return;
	const last = out.at(-1);
	if (last?.type === 'text') last.text += text;
	else out.push({ type: 'text', text });
}

function parse(text: string, data: Data, out: Segment[]) {
	const re = new RegExp(TOKEN);
	let last = 0;
	for (let m = re.exec(text); m; m = re.exec(text)) {
		pushText(out, text.slice(last, m.index));
		if (m[0] === '\n') out.push({ type: 'br' });
		else if (m[1]) out.push(placeholder(m[1], data));
		else if (m[2])
			out.push({
				type: 'element',
				element: ELEMENT_KEYS[m[3] as keyof typeof ELEMENT_KEYS],
				label: m[2]
			});
		else {
			const end = closing(text, m.index);
			if (end < 0) pushText(out, '[');
			else {
				const inner = text.slice(m.index + 1, end);
				if (/[<[]/.test(inner)) {
					pushText(out, '[');
					parse(inner, data, out);
					pushText(out, ']');
				} else out.push({ type: 'keyword', text: inner });
				re.lastIndex = end + 1;
			}
		}
		last = re.lastIndex;
	}
	pushText(out, text.slice(last));
}

export const fold = (s: string) => s.normalize('NFD').replace(/\p{M}/gu, '').toLocaleLowerCase();

export function matchKeyword<T extends { name: string }>(text: string, keywords: readonly T[]) {
	const needle = fold(text);
	let best: T | undefined;
	let length = 0;
	for (const keyword of keywords) {
		const name = fold(keyword.name);
		if (name === needle) return keyword;
		if (name.length > length && needle.startsWith(`${name} `)) {
			best = keyword;
			length = name.length;
		}
	}
	return best;
}

export function formatDescription(text: string, entry: { data: object }): Segment[] {
	const out: Segment[] = [];
	parse(text, entry.data as Data, out);
	return out;
}
