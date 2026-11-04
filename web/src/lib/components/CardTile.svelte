<script lang="ts">
	import { href, percent, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import CardArt from './CardArt.svelte';

	let {
		card,
		index = 0,
		stats
	}: { card: Schemas['CardBase']; index?: number; stats?: Schemas['CardStats'] } = $props();
</script>

<a
	href={href(`/cards/${card.id}`)}
	aria-label={card.name ?? card.id}
	class="flex h-full flex-col rounded-lg [contain-intrinsic-size:auto_360px] [content-visibility:auto] [overflow-clip-margin:16px] spring motion-safe:animate-rise motion-safe:hover:-translate-y-1 motion-safe:active:-translate-y-px motion-safe:active:scale-[.985]"
	style:--i={Math.min(index, 12)}
>
	<CardArt {card} interactive />
	{#if stats && stats.games > 0}
		<span
			class="flex flex-wrap justify-between gap-x-3 px-[8%] text-[11px] text-muted-foreground uppercase tabular-nums"
		>
			<span>{m.win_rate()} <b class="font-medium text-foreground">{percent(stats.winRate)}</b></span
			>
			<span>{m.games()} <b class="font-medium text-foreground">{stats.games}</b></span>
		</span>
	{/if}
</a>
