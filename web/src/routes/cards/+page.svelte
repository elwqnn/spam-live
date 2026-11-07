<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowRight02Icon } from '@hugeicons/core-free-icons';
	import { fold, href, queryHref } from '#lib';
	import { ELEMENTS, RARITIES } from '#lib/components/CardBadge.svelte';
	import CardList from '#lib/components/CardList.svelte';
	import { cardPreview } from '#lib/components/CardPreview.svelte';
	import FilterChips from '#lib/components/FilterChips.svelte';
	import FloatingElements from '#lib/components/FloatingElements.svelte';
	import TierBadge from '#lib/components/TierBadge.svelte';
	import { buttonVariants } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { selection, type Filter } from '#lib/filter.js';
	import TitleRow from '#lib/components/TitleRow.svelte';
	import TrendMenu from '#lib/components/TrendMenu.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import { TIERS, tierCards } from '#lib/tier.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const query = $derived(page.url.searchParams);
	const q = $derived(query.get('q') ?? '');
	const elements = $derived(selection(query.get('element')));
	const rarities = $derived(selection(query.get('rarity')).map((r) => `rarity-${r}`));
	const tiers = $derived(selection(query.get('tier')));
	const sort = $derived(query.get('sort') ?? 'order');
	const stats = $derived(new Map(data.stats.map((s) => [s.card, s])));
	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const top = $derived(tierCards(data.stats, 'S').flatMap((s) => catalog.get(s.card) ?? []));

	const cards = $derived.by(() => {
		const needle = fold(q.trim());
		const list = data.cards.filter(
			(c) =>
				(!elements.length || elements.includes(c.element)) &&
				(!rarities.length || rarities.includes(c.rarity)) &&
				(!tiers.length || tiers.includes(stats.get(c.id)?.tier ?? '')) &&
				(!needle || fold(c.name ?? c.id).includes(needle))
		);
		if (sort === 'name')
			return list.sort((a, b) => (a.name ?? a.id).localeCompare(b.name ?? b.id, getLocale()));
		if (sort === 'hp') return list.sort((a, b) => b.hp - a.hp || a.order - b.order);
		return list.sort((a, b) => a.order - b.order);
	});

	const count = (key: 'element' | 'rarity', value: string) =>
		data.cards.filter((c) => c[key] === value).length;

	const filters: Filter[] = $derived([
		{
			id: 'element',
			label: m.filter_element(),
			multiple: true,
			options: Object.entries(ELEMENTS).map(([value, label]) => ({
				value,
				label: label(),
				count: count('element', value),
				icon: `/art/ui/element_${value}.webp`
			}))
		},
		{
			id: 'rarity',
			label: m.filter_rarity(),
			multiple: true,
			options: Object.entries(RARITIES).map(([key, label]) => {
				const value = key.replace('rarity-', '');
				return {
					value,
					label: label(),
					count: count('rarity', key),
					icon: `/art/ui/card_rarity_${value}.webp`
				};
			})
		},
		{
			id: 'tier',
			label: m.filter_tier(),
			multiple: true,
			options: TIERS.map((value) => ({
				value,
				label: value,
				count: data.cards.filter((c) => stats.get(c.id)?.tier === value).length
			}))
		}
	]);

	const sorts: Filter = $derived({
		id: 'sort',
		label: m.sort(),
		default: 'order',
		options: [
			{ value: 'order', label: m.sort_order() },
			{ value: 'name', label: m.sort_name() },
			{ value: 'hp', label: m.sort_hp() }
		]
	});
</script>

<svelte:head>
	<title>{m.cards_title()} · {m.site_name()}</title>
</svelte:head>

<TitleRow class="flex flex-wrap items-end justify-between gap-4 row">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
		{m.cards_title()}
	</h1>
	<Input
		type="search"
		class="w-full sm:w-64"
		placeholder={m.search()}
		aria-label={m.search()}
		value={q}
		oninput={(e) =>
			goto(queryHref(page.url, { q: e.currentTarget.value || null }), {
				replace: true,
				reset: false
			})}
	/>
	<FloatingElements />
</TitleRow>

<section
	aria-labelledby="tiers-banner"
	class="flex flex-wrap items-center gap-x-8 gap-y-3 px-(--gutter) py-4 hatch"
>
	<div class="grid gap-0.5">
		<h2 id="tiers-banner" class="font-heading text-xl font-normal tracking-tight">
			{m.tiers_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.tiers_banner()}</p>
	</div>
	{#if top.length}
		<div class="flex items-start gap-2">
			<TierBadge tier="S" class="h-10 min-w-10 font-heading text-xl font-normal" />
			<ul class="flex flex-wrap gap-2">
				{#each top as card (card.id)}
					<li>
						<a
							class="block rounded-xs focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
							href={href(`/cards/${card.id}`)}
							{@attach cardPreview(card)}
							><img
								src="/art/avatar/{card.id}.webp"
								alt={card.name ?? card.id}
								loading="lazy"
								decoding="async"
								class="el-{card.element} size-10 rounded-xs border-2 border-(--el) bg-(--el) transition-colors hover:border-foreground"
							/></a
						>
					</li>
				{/each}
			</ul>
		</div>
	{/if}
	<a
		class={[buttonVariants({ variant: 'outline', size: 'sm' }), 'ml-auto']}
		href={href('/stats?tab=tiers')}
		>{m.tiers_banner_link()}
		<HugeiconsIcon icon={ArrowRight02Icon} aria-hidden="true" /></a
	>
</section>

<CardList {cards} {stats} {filters}>
	{#snippet actions()}
		<FilterChips filter={sorts} />
		<TrendMenu flat />
	{/snippet}
</CardList>
