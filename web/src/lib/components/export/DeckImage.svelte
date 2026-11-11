<script lang="ts">
	import { percent, winRateColor, type CardDetail, type Schemas } from '#lib';
	import CardArt from '@spam-live/ui/components/CardArt.svelte';
	import type { Size } from '#lib/export.js';
	import { m } from '#lib/paraglide/messages.js';
	import Stat from './Stat.svelte';

	let {
		deck,
		cards,
		size
	}: { deck: Schemas['DeckDetail']; cards: (CardDetail | null)[]; size: Size } = $props();

	const WIDTH = { wide: 400, square: 200, vertical: 400 };
	const GAP = 32;
	const CARD_ART_RATIO = 0.13 + 0.84 * (2048 / 1432);

	let height = $state(0);
	const stats = $derived(deck.stats.current);
	const rows = $derived(size.id === 'vertical' ? 2 : 1);
	const width = $derived(
		Math.max(
			0,
			Math.min(WIDTH[size.id], Math.floor((height - (rows - 1) * GAP) / rows / CARD_ART_RATIO))
		)
	);
</script>

<div class="flex h-full flex-col justify-center gap-[64px]">
	<div class="flex flex-wrap items-end justify-between gap-[48px]">
		<div class="grid min-w-0 gap-[16px]">
			<h2
				class="font-heading text-[80px] leading-[1.05] font-normal tracking-tight text-balance wrap-anywhere"
			>
				{deck.name}
			</h2>
			<p class="text-[32px] text-muted-foreground">{m.by_author({ name: deck.authorName })}</p>
		</div>
		<dl class="grid grid-cols-2 gap-x-[48px]">
			<Stat
				label={m.win_rate()}
				value={percent(stats.winRate)}
				color={winRateColor(stats.winRate)}
			/>
			<Stat label={m.games()} value={stats.games} />
		</dl>
	</div>
	<ul
		class="flex min-h-0 flex-1 flex-wrap content-center justify-center"
		style:gap="{GAP}px"
		bind:clientHeight={height}
	>
		{#each deck.cards as id, i (id)}
			{@const card = cards[i]}
			<li style:width="{width}px">
				{#if card}
					<CardArt {card} />
				{:else}
					<div
						class="mx-[8%] mt-[6%] mb-[7%] aspect-[1432/2048] rounded-sm border-2 border-dashed border-foreground/25"
					></div>
				{/if}
			</li>
		{/each}
	</ul>
</div>
