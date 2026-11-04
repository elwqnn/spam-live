<script lang="ts">
	import { href, percent, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import CardArt from './CardArt.svelte';
	import CardBadge from './CardBadge.svelte';
	import CardBadgeStack from './CardBadgeStack.svelte';

	let {
		card,
		index = 0,
		stats
	}: { card: Schemas['CardBase']; index?: number; stats?: Schemas['CardStats'] } = $props();

	let hover = $state(false);
	let focus = $state(false);
</script>

<a
	href={href(`/cards/${card.id}`)}
	class="el-{card.element} group/tile flex h-full flex-col overflow-hidden rounded-lg border bg-tile spring hover:border-(--el) motion-safe:animate-rise motion-safe:hover:-translate-y-1 motion-safe:active:-translate-y-px motion-safe:active:scale-[.985]"
	style:--i={Math.min(index, 12)}
	onpointerenter={() => (hover = true)}
	onpointerleave={() => (hover = false)}
	onfocus={() => (focus = true)}
	onblur={() => (focus = false)}
>
	<CardArt
		element={card.element}
		rarity={card.rarity}
		interactive
		animate={hover || focus}
		class="aspect-10/9 [--lift:1.05]"
	/>
	<div class="relative grid gap-2 bg-tile px-3 pt-2.5 pb-3 sm:px-3.5 sm:pt-3 sm:pb-3.5">
		<span class="text-[15px] leading-tight font-semibold">{card.name ?? card.id}</span>
		<CardBadgeStack>
			<CardBadge type="element" value={card.element} display="compact" />
			<CardBadge type="hp" value={card.hp} />
			<CardBadge type="income" value={card.goldIncome} />
		</CardBadgeStack>
		{#if stats && stats.games > 0}
			<span
				class="flex flex-wrap justify-between gap-x-3 text-[11px] text-muted-foreground uppercase tabular-nums"
			>
				<span
					>{m.win_rate()} <b class="font-medium text-foreground">{percent(stats.winRate)}</b></span
				>
				<span>{m.games()} <b class="font-medium text-foreground">{stats.games}</b></span>
			</span>
		{/if}
	</div>
</a>
