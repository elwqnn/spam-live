<script lang="ts">
	import FilterBar from '#lib/components/FilterBar.svelte';
	import TitleRow from '#lib/components/TitleRow.svelte';
	import { versionFilter } from '#lib/components/VersionSelect.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import Counters from './Counters.svelte';
	import Duos from './Duos.svelte';
	import History from './History.svelte';
	import Matchups from './Matchups.svelte';
	import MetaMap from './MetaMap.svelte';
	import Overview from './Overview.svelte';
	import Pace from './Pace.svelte';
	import Turns from './Turns.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
</script>

<svelte:head>
	<title>{m.stats_title()} · {m.site_name()}</title>
</svelte:head>

<TitleRow class="grid gap-1 row">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
		{m.stats_title()}
	</h1>
	<p class="text-sm text-muted-foreground">{m.stats_intro()}</p>
</TitleRow>

<div class="px-(--gutter) py-3">
	<FilterBar filters={[versionFilter(data.versions)]} />
</div>

<Overview data={data.overview} />

<MetaMap data={data.stats} {catalog} />

<Matchups data={data.matchups} {catalog} />

<Counters data={data.counters} against={data.against} cards={data.cards} {catalog} />

<Duos data={data.duos} {catalog} />

<Pace data={data.pace} {catalog} />

<Turns data={data.turns} {catalog} />

<History data={data.history} />
