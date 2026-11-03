<script lang="ts">
	import { href, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import CardArt from './CardArt.svelte';
	import { ELEMENTS } from './ElementBadge.svelte';

	let { card, index = 0 }: { card: Schemas['CardBase']; index?: number } = $props();
</script>

<a
	href={href(`/cards/${card.id}`)}
	class="el-{card.element} group/tile flex aspect-5/7 flex-col overflow-hidden rounded-lg border bg-tile spring hover:border-(--el) motion-safe:animate-rise motion-safe:hover:-translate-y-1 motion-safe:active:-translate-y-px motion-safe:active:scale-[.985]"
	style:--i={Math.min(index, 12)}
>
	<CardArt element={card.element} rarity={card.rarity} class="min-h-0 flex-1" />
	<div class="grid gap-1.5 px-3 pt-2.5 pb-3 sm:px-3.5 sm:pt-3 sm:pb-3.5">
		<span class="mono-label text-[10px] text-el"
			>{ELEMENTS[card.element]?.[0]() ?? card.element}</span
		>
		<span class="text-[15px] leading-tight font-semibold">{card.name ?? card.id}</span>
		<span class="flex flex-wrap gap-x-3 font-mono text-[11px] text-muted-foreground uppercase">
			<span>{m.hp()} <b class="font-medium text-foreground">{card.hp}</b></span>
			<span>{m.income()} <b class="font-medium text-foreground">{card.goldIncome}</b></span>
		</span>
	</div>
</a>
