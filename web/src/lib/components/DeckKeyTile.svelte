<script lang="ts">
	import { percent, type Schemas } from '#lib';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { Reason } from '#lib/promote.js';
	import CardBadge from './CardBadge.svelte';
	import CardBadgeStack from './CardBadgeStack.svelte';
	import DeckCards from './DeckCards.svelte';
	import LiveLines from './LiveLines.svelte';

	let {
		deck,
		rank,
		catalog,
		reasons = [],
		large = false
	}: {
		deck: Schemas['DeckKeyStats'];
		rank: number;
		catalog: ReadonlyMap<string, Schemas['CardBase']>;
		reasons?: Reason[];
		large?: boolean;
	} = $props();

	const order = (id: string) => catalog.get(id)?.order ?? Number.MAX_SAFE_INTEGER;
	const cards = $derived([...deck.cards].sort((a, b) => order(a) - order(b)));
	const mix = $derived(
		cards.reduce((map, id) => {
			const element = catalog.get(id)?.element;
			return element ? map.set(element, (map.get(element) ?? 0) + 1) : map;
		}, new Map<string, number>())
	);
</script>

<li
	class={[
		'flex flex-col gap-3 px-(--gutter) py-4 transition-colors [contain-intrinsic-size:auto_200px] [content-visibility:auto] sm:py-5',
		reasons.length ? 'relative isolate bg-card' : 'hover:bg-row',
		large && 'lg:col-span-2 lg:row-span-2 lg:[contain-intrinsic-size:auto_480px]'
	]}
>
	{#if reasons.length}
		<LiveLines line={0.5} speed={1 / 3} class="opacity-7 [:hover>&]:opacity-9" />
	{/if}
	<div class="flex flex-wrap items-center gap-x-3 gap-y-1">
		<span class="text-sm text-muted-foreground tabular-nums">#{String(rank).padStart(2, '0')}</span>
		<CardBadgeStack>
			{#each mix as [element, count] (element)}
				<CardBadge type="element" value={element} display="compact"
					><span class="tabular-nums">×{count}</span></CardBadge
				>
			{/each}
		</CardBadgeStack>
		{#each reasons as reason (reason)}
			<Badge variant="outline">{reason === 'votes' ? m.top_voted() : m.strong_win_rate()}</Badge>
		{/each}
	</div>
	<DeckCards {cards} {catalog} {large} />
	<dl class="mt-auto flex flex-wrap gap-x-6 gap-y-2 text-[13px] tabular-nums">
		<div class="grid gap-0.5">
			<dt class="eyebrow text-[10px] text-muted-foreground">{m.games()}</dt>
			<dd class={{ 'lg:text-2xl': large }}>{deck.games}</dd>
		</div>
		<div class="grid gap-0.5">
			<dt class="eyebrow text-[10px] text-muted-foreground">{m.win_rate()}</dt>
			<dd class={{ 'lg:text-2xl': large }}>{percent(deck.winRate)}</dd>
		</div>
	</dl>
</li>
