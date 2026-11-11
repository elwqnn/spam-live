<script lang="ts">
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';
	import { queryHref, type PageData } from '$host';
	import FilterMenu from '#ui/components/FilterMenu.svelte';
	import TitleRow from '#ui/components/TitleRow.svelte';
	import VersionSelect from '#ui/components/VersionSelect.svelte';
	import type { Filter } from '#ui/filter.js';
	import { STAR_BRACKETS, starLabel } from '#ui/stars.js';
	import { buttonVariants } from '#ui/components/ui/button/index.js';
	import { m } from '$paraglide/messages.js';
	import Counters from './stats/Counters.svelte';
	import Duos from './stats/Duos.svelte';
	import History from './stats/History.svelte';
	import MetaMap from './stats/MetaMap.svelte';
	import Overview from './stats/Overview.svelte';
	import Pace from './stats/Pace.svelte';
	import Strength from './stats/Strength.svelte';
	import Tiers from './stats/Tiers.svelte';
	import type { statsData } from './data.js';

	let {
		data,
		exportImage
	}: { data: Awaited<ReturnType<typeof statsData>>; exportImage?: Snippet } = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const tabs = [
		['meta', m.stats_tab_meta],
		['counters', m.stats_tab_counters],
		['duos', m.stats_tab_duos],
		['length', m.stats_tab_length],
		['trends', m.stats_tab_trends]
	] as const;
	const stars: Filter = $derived({
		id: 'minStars',
		label: m.stars_filter(),
		default: 'all',
		options: [
			{ value: 'all', label: m.stars_all() },
			...STAR_BRACKETS.map((b) => ({
				value: String(b),
				label: starLabel(b, (page.data as PageData).names?.ranks)
			}))
		]
	});
	const label = $derived(tabs.find(([key]) => key === data.tab)?.[1]() ?? '');
	const link = (active: boolean) =>
		buttonVariants({ variant: active ? 'secondary' : 'ghost', size: 'sm' });
</script>

<svelte:head>
	<title>{label} · {m.stats_title()} · {m.site_name()}</title>
</svelte:head>

<TitleRow class="grid gap-1 row">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
		{m.stats_title()}
	</h1>
	<p class="text-sm text-muted-foreground">{m.stats_intro()}</p>
</TitleRow>

<div class="flex flex-wrap items-center gap-x-4 gap-y-2 px-(--gutter) py-3">
	<nav aria-label={m.stats_tabs()} class="flex flex-wrap gap-1">
		{#each tabs as [key, name] (key)}
			<a
				class={link(data.tab === key)}
				aria-current={data.tab === key ? 'page' : undefined}
				href={queryHref(page.url, { tab: key === 'meta' ? null : key })}>{name()}</a
			>
		{/each}
	</nav>
	<div class="ml-auto">
		{#if data.tab === 'trends'}
			<span class="text-sm text-muted-foreground">{m.all_versions()}</span>
		{:else}
			<div role="group" aria-label={m.filters()} class="flex flex-wrap items-center gap-2">
				<VersionSelect versions={data.versions} />
				<FilterMenu filter={stars} selected={data.minStars?.toString() ?? null} />
			</div>
		{/if}
	</div>
</div>

{#if data.tab === 'meta' && data.overview}
	<Overview data={data.overview} />
	<Tiers data={data.stats} {catalog} {exportImage} />
	<MetaMap data={data.stats} {catalog} />
	<Strength data={data.stats} {catalog} />
{:else if data.tab === 'counters'}
	<Counters data={data.counters} against={data.against} cards={data.cards} {catalog} />
{:else if data.tab === 'duos'}
	<Duos data={data.duos} stats={data.stats} {catalog} />
{:else if data.tab === 'length' && data.pace}
	<Pace data={data.pace} {catalog} />
{:else if data.tab === 'trends' && data.history}
	<History data={data.history} />
{/if}
