<script lang="ts">
	import { href, percent, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import CardArt from './CardArt.svelte';

	let { card, stats }: { card: Schemas['CardBase']; stats?: Schemas['CardStats'] } = $props();
</script>

<a
	href={href(`/cards/${card.id}`)}
	aria-label={card.name ?? card.id}
	class="flex h-full flex-col p-2 pb-3 transition-colors outline-none [contain-intrinsic-size:auto_360px] [content-visibility:auto] [overflow-clip-margin:16px] hover:bg-row focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset sm:p-3 sm:pb-4"
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
