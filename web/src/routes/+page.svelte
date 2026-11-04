<script lang="ts">
	import { href } from '#lib';
	import Arrow from '#lib/components/Arrow.svelte';
	import Author from '#lib/components/Author.svelte';
	import CardStack from '#lib/components/CardStack.svelte';
	import DeckCards from '#lib/components/DeckCards.svelte';
	import LiveLines from '#lib/components/LiveLines.svelte';
	import ShaderBackground from '#lib/components/ShaderBackground.svelte';
	import WinRate from '#lib/components/WinRate.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Table from '#lib/components/ui/table/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
	const count = $derived(data.cards.length);
	const line = 'border-[#fbf8f0]/35 bg-[#171715]/35 hover:border-[#fbf8f0]/70';
</script>

<svelte:head>
	<meta name="description" content={m.site_tagline()} />
</svelte:head>

{#snippet heading(index: string, eyebrow: string, title: string)}
	<div class="grid gap-1.5">
		<span class="eyebrow tracking-[.08em] text-muted-foreground">{index} · {eyebrow}</span>
		<h2 class="font-heading text-3xl/tight font-normal tracking-tight">{title}</h2>
	</div>
{/snippet}

<section class="relative isolate text-[#fbf8f0]">
	<ShaderBackground />
	<div
		class="relative flex flex-col items-center gap-7 px-4 pt-24 pb-28 text-center sm:px-6 sm:pt-32 sm:pb-36"
	>
		<h1
			class="max-w-4xl font-heading text-5xl/none font-normal tracking-[-0.025em] text-balance sm:text-7xl/none"
		>
			{m.home_title()}
		</h1>
		<p class="max-w-xl text-lg">{m.home_subtitle()}</p>
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
		<form
			method="GET"
			action={href('/cards')}
			role="search"
			class="flex h-13 w-full max-w-145 items-center gap-3.5 rounded-sm border border-[#fbf8f0]/16 bg-[#171715]/70 pr-1.5 pl-4.5 backdrop-blur-sm transition-colors focus-within:border-[#fbf8f0]/60 focus-within:bg-[#171715]/80"
		>
			<span aria-hidden="true" class="font-mono text-sm">/</span>
			<input
				type="search"
				name="q"
				aria-label={m.search()}
				placeholder={m.home_search({ count })}
				class="h-full min-w-0 flex-1 bg-transparent text-sm text-[#fbf8f0] outline-none placeholder:text-[#fbf8f0]/75"
			/>
			<button
				type="submit"
				aria-label={m.search()}
				class="grid size-10 shrink-0 place-items-center rounded-sm transition-colors hover:bg-[#fbf8f0]/10 focus-visible:ring-1 focus-visible:ring-[#fbf8f0]/60"
				><Arrow class="size-4" /></button
			>
		</form>
	</div>
</section>

<section class="rows">
	<div class="flex flex-wrap items-end gap-4 row">
		{@render heading('01', m.home_meta(), m.home_top_decks())}
		<a
			class="mb-1.5 ml-auto eyebrow text-muted-foreground transition-colors hover:text-foreground"
			href={href('/stats')}>{m.see_all_stats()} <span aria-hidden="true">→</span></a
		>
	</div>
	{#if data.top.length}
		<div>
			<Table.Root>
				<Table.Header>
					<Table.Row class="hover:bg-transparent">
						<Table.Head class="w-12 pl-4 sm:pl-6">#</Table.Head>
						<Table.Head>{m.deck()}</Table.Head>
						<Table.Head class="hidden text-right sm:table-cell">{m.games()}</Table.Head>
						<Table.Head class="pr-4 text-right sm:pr-6">{m.win_rate()}</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each data.top as deck, i (deck.cards.join())}
						<Table.Row>
							<Table.Cell class="pl-4 text-xs text-muted-foreground tabular-nums sm:pl-6"
								>{String(i + 1).padStart(2, '0')}</Table.Cell
							>
							<Table.Cell class="py-3.5 whitespace-normal">
								<DeckCards cards={deck.cards} {catalog} />
							</Table.Cell>
							<Table.Cell class="hidden text-right text-[13px] tabular-nums sm:table-cell"
								>{deck.games}</Table.Cell
							>
							<Table.Cell class="pr-4 sm:pr-6"><WinRate value={deck.winRate} /></Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</div>
	{:else}
		<p class="row text-sm text-muted-foreground">{m.home_no_top_decks()}</p>
	{/if}
</section>

<section class="rows">
	<div class="row">
		{@render heading('02', m.home_community(), m.home_recent_decks())}
	</div>
	{#if data.recent.length}
		<CardStack>
			{#each data.recent as deck (deck.id)}
				<li class="grid gap-2.5 px-4 py-4 transition-colors hover:bg-row sm:px-6">
					<div class="flex flex-wrap items-center gap-x-3 gap-y-1">
						<a
							class="text-[15px] font-medium wrap-anywhere hover:underline"
							href={href(`/decks/${deck.id}`)}>{deck.name}</a
						>
						<span class="flex min-w-0 text-sm text-muted-foreground">
							<Author name={deck.authorName} avatar={deck.authorAvatar} />
						</span>
						<span class="ml-auto eyebrow text-muted-foreground"
							>{m.votes()} <b class="font-medium text-foreground">{deck.votes}</b></span
						>
					</div>
					<DeckCards cards={deck.cards} {catalog} />
				</li>
			{/each}
		</CardStack>
	{:else}
		<p class="row text-sm text-muted-foreground">{m.home_no_recent_decks()}</p>
	{/if}
</section>

<section
	class="relative isolate flex flex-wrap items-end justify-between gap-6 bg-card row sm:py-12"
>
	<LiveLines line={0.5} speed={1 / 3} class="opacity-7 [:hover>&]:opacity-9" />
	<div class="grid max-w-xl gap-3">
		{@render heading('03', m.home_contribute(), m.import_title())}
		<p class="text-muted-foreground">{m.import_text()}</p>
	</div>
	<Button size="lg" href={href('/import')}>{m.nav_import()}<Arrow /></Button>
</section>
