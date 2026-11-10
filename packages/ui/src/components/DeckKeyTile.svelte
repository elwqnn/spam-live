<script lang="ts">
	import { percent, type Schemas } from '#ui';
	import { m } from '$paraglide/messages.js';
	import type { Reason } from '#ui/promote.js';
	import CardBadge from './CardBadge.svelte';
	import CardBadgeStack from './CardBadgeStack.svelte';
	import DeckCards from './DeckCards.svelte';
	import LiveLines from './LiveLines.svelte';

	let {
		deck,
		rank,
		catalog,
		reasons = [],
		large = false,
		stats
	}: {
		deck: Schemas['DeckKeyStats'];
		rank: number;
		catalog: ReadonlyMap<string, Schemas['CardBase']>;
		reasons?: Reason[];
		large?: boolean;
		stats?: ReadonlyMap<string, Schemas['CardStats']>;
	} = $props();

	const order = (id: string) => catalog.get(id)?.order ?? Number.MAX_SAFE_INTEGER;
	const cards = $derived([...deck.cards].sort((a, b) => order(a) - order(b)));
	const mix = $derived(
		cards.reduce((map, id) => {
			const element = catalog.get(id)?.element;
			return element ? map.set(element, (map.get(element) ?? 0) + 1) : map;
		}, new Map<string, number>())
	);

	let tilted: HTMLElement | null = null;

	function rest() {
		if (!tilted) return;
		tilted.style.transform = '';
		delete tilted.dataset.tilt;
		tilted = null;
	}

	function move(e: PointerEvent) {
		if (e.pointerType !== 'mouse' || matchMedia('(prefers-reduced-motion: reduce)').matches) {
			rest();
			return;
		}
		const link = (e.target as Element).closest<HTMLElement>('a');
		if (link !== tilted) rest();
		if (!link?.parentElement) return;
		const rect = link.parentElement.getBoundingClientRect();
		const x = Math.min(1, Math.max(0, (e.clientX - rect.left) / link.offsetWidth));
		const y = Math.min(1, Math.max(0, (e.clientY - rect.top) / link.offsetHeight));
		link.style.transform = `perspective(800px) rotateX(${(0.5 - y) * 16}deg) rotateY(${(x - 0.5) * 16}deg)`;
		link.dataset.tilt = '';
		tilted = link;
	}
</script>

<li
	class={[
		'flex flex-col gap-3 px-(--gutter) py-4 transition-colors [contain-intrinsic-size:auto_200px] [content-visibility:auto] sm:py-5',
		reasons.length ? 'relative isolate bg-card' : 'hover:bg-row',
		large && 'lg:col-span-2 lg:row-span-2 lg:[contain-intrinsic-size:auto_480px]'
	]}
>
	{#if reasons.length}
		<LiveLines line={1} speed={1 / 3} class="opacity-12 [:hover>&]:opacity-12" />
	{/if}
	<div class="flex flex-wrap items-center gap-x-3 gap-y-1">
		<span class="text-sm text-muted-foreground tabular-nums">#{String(rank).padStart(2, '0')}</span>
		<CardBadgeStack class="ml-auto justify-end">
			{#each mix as [element, count] (element)}
				<CardBadge type="element" value={element} display="compact"
					><span class="tabular-nums">{count}</span></CardBadge
				>
			{/each}
		</CardBadgeStack>
	</div>
	<div
		class={[
			'contents',
			large &&
				'[&_a]:duration-700 [&_a]:ease-(--spring) [&_a]:motion-safe:transition-transform [&_a[data-tilt]]:duration-100 [&_a[data-tilt]]:ease-out'
		]}
		role="presentation"
		onpointermove={large ? move : undefined}
		onpointerleave={large ? rest : undefined}
		onpointercancel={large ? rest : undefined}
	>
		<DeckCards {cards} {catalog} {large} {stats} />
	</div>
	{#if large}
		<p class="mt-auto text-sm text-muted-foreground">
			<span class="shader-text text-4xl font-medium tracking-tight tabular-nums lg:text-5xl"
				>{percent(deck.winRate)}</span
			>
			{m.top_deck_of()}
			<span class="font-semibold text-foreground">{m.top_deck_win_rate()}</span>
			{m.top_deck_over()}
			<span class="text-xl text-foreground tabular-nums lg:text-3xl">{deck.games}</span>
			<span class="font-semibold text-foreground">{m.games_unit({ count: deck.games })}</span>
		</p>
	{:else}
		<dl class="mt-auto flex flex-wrap gap-x-6 gap-y-2 text-[13px] tabular-nums">
			<div class="grid gap-0.5">
				<dt class="eyebrow text-[10px] text-muted-foreground">{m.win_rate()}</dt>
				<dd class={{ 'shader-text': reasons.includes('winrate') }}>{percent(deck.winRate)}</dd>
			</div>
			<div class="grid gap-0.5">
				<dt class="eyebrow text-[10px] text-muted-foreground">{m.games()}</dt>
				<dd>{deck.games}</dd>
			</div>
		</dl>
	{/if}
</li>
