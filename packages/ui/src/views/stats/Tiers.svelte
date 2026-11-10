<script lang="ts">
	import { percent, winRateColor, type CardMap, type Schemas } from '#ui';
	import { art, href } from '$host';
	import { cardPreview } from '#ui/components/CardPreview.svelte';
	import TierBadge from '#ui/components/TierBadge.svelte';
	import { m } from '$paraglide/messages.js';
	import { TIERS, tierCards } from '#ui/tier.js';

	let { data, catalog }: { data: Schemas['CardStats'][]; catalog: CardMap } = $props();

	const tiers = $derived(TIERS.map((tier) => ({ tier, cards: tierCards(data, tier) })));
</script>

{#snippet avatars(cards: Schemas['CardStats'][])}
	<ul class="flex flex-wrap gap-3">
		{#each cards as s (s.card)}
			{@const card = catalog.get(s.card)}
			<li class="grid w-12 justify-items-center gap-1">
				{#if card}
					<a
						class="block rounded-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
						href={href(`/cards/${s.card}`)}
						{@attach cardPreview(card)}
						><img
							src={art(`avatar/${s.card}.webp`)}
							alt={card.name ?? s.card}
							loading="lazy"
							decoding="async"
							class="el-{card.element} size-12 rounded-xs border-2 border-(--el) bg-(--el) transition-colors hover:border-foreground"
						/></a
					>
				{:else}
					<span
						class="grid size-12 place-items-center rounded-xs border border-dashed border-foreground/25 font-mono text-[10px] text-muted-foreground"
						><span class="sr-only">{m.unknown_card()}</span> {s.card.slice(0, 4)}</span
					>
				{/if}
				<span class="text-xs tabular-nums" style:color={winRateColor(s.winRate)}
					><span class="sr-only">{m.win_rate()}</span> {percent(s.winRate)}</span
				>
			</li>
		{/each}
	</ul>
{/snippet}

<section id="tiers" aria-labelledby="tiers-title" class="sm:scroll-mt-(--head)">
	<div class="grid gap-1 section-head">
		<h2 id="tiers-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.tiers_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.tiers_text()}</p>
	</div>
	{#if data.length}
		<ol class="divide-y">
			{#each tiers as { tier, cards } (tier)}
				<li class="flex min-h-20 gap-4 px-(--gutter) sm:gap-6">
					<TierBadge {tier} stretch />
					<div class="min-w-0 flex-1 py-3">
						{#if cards.length}{@render avatars(cards)}{/if}
					</div>
				</li>
			{/each}
		</ol>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.no_stats()}</p>
	{/if}
</section>
