<script lang="ts">
	import { href } from '#lib';
	import Arrow from '#lib/components/Arrow.svelte';
	import DeckKeyTile from '#lib/components/DeckKeyTile.svelte';
	import DeckTile from '#lib/components/DeckTile.svelte';
	import LiveLines from '#lib/components/LiveLines.svelte';
	import ShaderBackground from '#lib/components/ShaderBackground.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const line = 'border-[#fbf8f0]/35 bg-[#171715]/35 hover:border-[#fbf8f0]/70';
</script>

<svelte:head>
	<meta name="description" content={m.site_tagline()} />
</svelte:head>

{#snippet heading(title: string)}
	<h2 class="font-heading text-3xl/tight font-normal tracking-tight">{title}</h2>
{/snippet}

<section class="relative isolate text-[#fbf8f0]">
	<ShaderBackground />
	<div
		class="relative flex flex-col items-center gap-5 px-4 pt-16 pb-20 text-center sm:px-6 sm:pt-20 sm:pb-24"
	>
		<h1
			class="font-heading text-3xl/none font-normal tracking-[-0.025em] text-balance sm:text-4xl/none xl:text-5xl/none"
		>
			{m.home_title()}
		</h1>
		<p class="max-w-2xl text-lg text-balance">{m.home_subtitle()}</p>
		<div class="flex flex-wrap justify-center gap-3">
			<Button
				size="lg"
				href={href('/decks')}
				class="border-[#f2ead3] bg-[#f2ead3] text-[#171715] hover:bg-[#fbf4df]"
				>{m.home_browse_decks()}<Arrow /></Button
			>
			<Button size="lg" variant="outline" href={href('/import')} class={line}
				>{m.import_title()}</Button
			>
		</div>
	</div>
</section>

<section>
	<div class="flex flex-wrap items-end gap-4 section-head">
		{@render heading(m.home_top_decks())}
		<a
			class="mb-1.5 ml-auto eyebrow text-muted-foreground transition-colors hover:text-foreground"
			href={href('/stats')}>{m.see_all_stats()} <span aria-hidden="true">→</span></a
		>
	</div>
	{#if data.top.length}
		<div>
			<ul class="cells grid sm:grid-cols-2 lg:grid-cols-3">
				{#each data.top as deck, i (deck.cards.join())}
					<DeckKeyTile {deck} rank={i + 1} {catalog} />
				{/each}
			</ul>
		</div>
	{:else}
		<p class="px-4 pb-8 text-sm text-muted-foreground sm:px-6 sm:pb-10">{m.home_no_top_decks()}</p>
	{/if}
</section>

<section>
	<div class="section-head">
		{@render heading(m.home_recent_decks())}
	</div>
	{#if data.recent.length}
		<div>
			<ul class="cells grid sm:grid-cols-2 lg:grid-cols-3">
				{#each data.recent as deck (deck.id)}
					<DeckTile {deck} {catalog} />
				{/each}
			</ul>
		</div>
	{:else}
		<p class="px-4 pb-8 text-sm text-muted-foreground sm:px-6 sm:pb-10">
			{m.home_no_recent_decks()}
		</p>
	{/if}
</section>

<section
	class="relative isolate flex flex-wrap items-end justify-between gap-6 bg-card row sm:py-12"
>
	<LiveLines line={0.5} speed={1 / 3} class="opacity-7 [:hover>&]:opacity-9" />
	<div class="grid max-w-xl gap-3">
		{@render heading(m.import_title())}
		<p class="text-muted-foreground">{m.import_text()}</p>
	</div>
	<Button size="lg" href={href('/import')}>{m.nav_import()}<Arrow /></Button>
</section>
