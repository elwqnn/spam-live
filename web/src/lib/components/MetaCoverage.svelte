<script lang="ts" module>
	const RAIL = 8;
</script>

<script lang="ts">
	import { href, percent, type Schemas } from '#lib';
	import { coverage, type Count } from '#lib/coverage.js';
	import { m } from '#lib/paraglide/messages.js';
	import CardArt from './CardArt.svelte';
	import { ELEMENTS, RARITIES } from './CardBadge.svelte';
	import RarityMark from './RarityMark.svelte';

	let { cards, stats }: { cards: Schemas['CardBase'][]; stats: Schemas['CardStats'][] } = $props();

	const result = $derived(coverage(cards, stats));
	const ordered = (group: Record<string, Count>, known: object) =>
		[...new Set([...Object.keys(known), ...Object.keys(group)])]
			.filter((key) => key in group)
			.map((key) => [key, group[key]] as const);
	const rail = $derived(result.unplayed.slice(0, RAIL));
	const more = $derived(result.unplayed.length - rail.length);
</script>

{#snippet bar(share: number, fill: string)}
	<span class="block h-1.5 overflow-hidden rounded-[2px] bg-foreground/6">
		<span
			class={['block h-full origin-left motion-safe:animate-grow', fill]}
			style:width="{share * 100}%"
		></span>
	</span>
{/snippet}

{#snippet row(key: string, count: Count, mark: 'rarity' | 'element')}
	<li class="grid grid-cols-[8.5rem_minmax(0,1fr)_3.5rem] items-center gap-3 text-sm">
		{#if mark === 'rarity'}
			<span class="flex min-w-0 items-center gap-2">
				<RarityMark rarity={key} tooltip={false} class="w-10 text-muted-foreground" />
				<span class="truncate">{RARITIES[key]?.() ?? key}</span>
			</span>
			{@render bar(count.played / count.total, 'bg-foreground/70')}
		{:else}
			<span class="truncate">{ELEMENTS[key]?.[0]() ?? key}</span>
			{@render bar(count.played / count.total, `el-${key} bg-(--el)`)}
		{/if}
		<span class="text-right text-[13px] text-muted-foreground tabular-nums"
			><b class="font-medium text-foreground">{count.played}</b> / {count.total}</span
		>
	</li>
{/snippet}

<section class="grid gap-6 rounded-xl border bg-card p-5 sm:p-6" aria-labelledby="coverage-title">
	<div class="grid gap-1">
		<h2 id="coverage-title" class="eyebrow text-muted-foreground">{m.meta_coverage()}</h2>
		<p class="text-sm text-muted-foreground">{m.meta_coverage_text()}</p>
	</div>
	{#if result.played}
		<div class="grid gap-8 md:grid-cols-[minmax(0,1fr)_minmax(0,1.3fr)]">
			<div class="grid content-start gap-5">
				<div class="grid gap-2">
					<span class="text-5xl font-medium tracking-tight tabular-nums"
						>{percent(result.share)}</span
					>
					{@render bar(result.share, 'bg-foreground/70')}
					<span class="text-sm text-muted-foreground tabular-nums"
						>{m.cards_played({ played: result.played, total: result.total })}</span
					>
				</div>
				{#if rail.length}
					<div class="grid gap-2">
						<h3 class="eyebrow text-[10px] text-muted-foreground">{m.never_played()}</h3>
						<ul class="flex flex-wrap items-center gap-1.5">
							{#each rail as card (card.id)}
								<li>
									<a
										href={href(`/cards/${card.id}`)}
										title={card.name ?? card.id}
										class="block rounded-xs spring motion-safe:hover:-translate-y-0.5"
									>
										<CardArt
											element={card.element}
											rarity={card.rarity}
											tag={false}
											class="aspect-5/7 w-9 rounded-xs"
										/>
										<span class="sr-only">{card.name ?? card.id}</span>
									</a>
								</li>
							{/each}
							{#if more}
								<li class="px-1 text-sm text-muted-foreground tabular-nums">
									<span aria-hidden="true">+{more}</span>
									<span class="sr-only">{m.more_cards({ n: more })}</span>
								</li>
							{/if}
						</ul>
					</div>
				{/if}
			</div>
			<div class="grid content-start gap-6 sm:grid-cols-2 md:grid-cols-1 lg:grid-cols-2">
				<div class="grid content-start gap-2.5">
					<h3 class="eyebrow text-[10px] text-muted-foreground">{m.by_rarity()}</h3>
					<ul class="grid gap-2">
						{#each ordered(result.rarities, RARITIES) as [key, count] (key)}
							{@render row(key, count, 'rarity')}
						{/each}
					</ul>
				</div>
				<div class="grid content-start gap-2.5">
					<h3 class="eyebrow text-[10px] text-muted-foreground">{m.by_element()}</h3>
					<ul class="grid gap-2">
						{#each ordered(result.elements, ELEMENTS) as [key, count] (key)}
							{@render row(key, count, 'element')}
						{/each}
					</ul>
				</div>
			</div>
		</div>
	{:else}
		<p class="text-sm text-muted-foreground">{m.no_coverage()}</p>
	{/if}
</section>
