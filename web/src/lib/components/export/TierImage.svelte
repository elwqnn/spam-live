<script lang="ts">
	import { art, type CardMap, type Schemas } from '#lib';
	import TierBadge from '@spam-live/ui/components/TierBadge.svelte';
	import { TIERS, tierCards } from '@spam-live/ui/tier.js';
	import { BADGE, tierLayout } from '#lib/export.js';
	import { m } from '#lib/paraglide/messages.js';

	let { stats, catalog }: { stats: Schemas['CardStats'][]; catalog: CardMap } = $props();

	let width = $state(0);
	let height = $state(0);
	const tiers = $derived(
		TIERS.map((tier) => ({
			tier,
			cards: tierCards(stats, tier).flatMap((s) => catalog.get(s.card) ?? [])
		}))
	);
	const layout = $derived(
		tierLayout(
			tiers.map((t) => t.cards.length),
			width,
			height
		)
	);
</script>

<div class="flex h-full flex-col gap-[40px]">
	<h2 class="font-heading text-[80px] leading-none font-normal tracking-tight">
		{m.tiers_title()}
	</h2>
	<ol class="min-h-0 flex-1 divide-y" bind:clientWidth={width} bind:clientHeight={height}>
		{#each tiers as { tier, cards } (tier)}
			<li class="flex" style:gap="{layout.gap}px" style:padding-block="{layout.gap}px">
				<div class="flex shrink-0" style:width="{BADGE}px" style:height="{BADGE}px">
					<TierBadge {tier} class="size-full rounded-sm text-[48px]" />
				</div>
				<ul class="flex min-w-0 flex-1 flex-wrap content-start" style:gap="{layout.gap}px">
					{#each cards as card (card.id)}
						<li>
							<img
								src={art(`avatar/${card.id}.webp`)}
								alt={card.name ?? card.id}
								class="el-{card.element} block rounded-sm border-(--el) bg-(--el)"
								style:width="{layout.size}px"
								style:height="{layout.size}px"
								style:border-width="{Math.max(2, Math.round(layout.size / 24))}px"
							/>
						</li>
					{/each}
				</ul>
			</li>
		{/each}
	</ol>
</div>
