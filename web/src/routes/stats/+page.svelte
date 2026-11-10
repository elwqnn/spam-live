<script lang="ts">
	import { page } from '$app/state';
	import { queryHref } from '#lib';
	import FilterBar from '#lib/components/FilterBar.svelte';
	import TitleRow from '#lib/components/TitleRow.svelte';
	import { versionFilter } from '#lib/components/VersionSelect.svelte';
	import { buttonVariants } from '#lib/components/ui/button/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import Counters from './Counters.svelte';
	import Duos from './Duos.svelte';
	import History from './History.svelte';
	import MetaMap from './MetaMap.svelte';
	import Overview from './Overview.svelte';
	import Pace from './Pace.svelte';
	import Strength from './Strength.svelte';
	import Tiers from './Tiers.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const tabs = [
		['meta', m.stats_tab_meta],
		['counters', m.stats_tab_counters],
		['duos', m.stats_tab_duos],
		['length', m.stats_tab_length],
		['trends', m.stats_tab_trends]
	] as const;
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
			<FilterBar filters={[versionFilter(data.versions)]} />
		{/if}
	</div>
</div>

{#if data.tab === 'meta' && data.overview}
	<Overview data={data.overview} />
	<Tiers data={data.stats} {catalog} />
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
