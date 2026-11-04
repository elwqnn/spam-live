<script lang="ts">
	import { href, type Schemas } from '#lib';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import Author from './Author.svelte';
	import DeckCards from './DeckCards.svelte';
	import WinRate from './WinRate.svelte';

	let {
		deck,
		catalog
	}: { deck: Schemas['DeckItem']; catalog: ReadonlyMap<string, Schemas['CardBase']> } = $props();
</script>

<li
	class="grid gap-x-6 gap-y-3 px-5 py-4 transition-colors hover:bg-row md:grid-cols-[minmax(0,1fr)_auto] md:items-center"
>
	<div class="grid min-w-0 gap-2.5">
		<div class="flex flex-wrap items-center gap-x-3 gap-y-1">
			<a
				class="text-[15px] font-medium wrap-anywhere hover:underline"
				href={href(`/decks/${deck.id}`)}>{deck.name}</a
			>
			{#if deck.status === 'draft'}<Badge variant="secondary">{m.draft()}</Badge>{/if}
			<span class="flex min-w-0 text-sm text-muted-foreground">
				<Author name={deck.authorName} avatar={deck.authorAvatar} />
			</span>
		</div>
		<DeckCards cards={deck.cards} {catalog} />
	</div>
	<dl class="flex flex-wrap items-center gap-x-6 gap-y-2 text-[13px] tabular-nums md:justify-end">
		<div class="grid gap-0.5">
			<dt class="eyebrow text-[10px] text-muted-foreground">{m.votes()}</dt>
			<dd>{deck.votes}</dd>
		</div>
		<div class="grid gap-0.5">
			<dt class="eyebrow text-[10px] text-muted-foreground">{m.games()}</dt>
			<dd>{deck.games}</dd>
		</div>
		<div class="grid gap-0.5">
			<dt class="text-right eyebrow text-[10px] text-muted-foreground">{m.win_rate()}</dt>
			<dd><WinRate value={deck.winRate} /></dd>
		</div>
	</dl>
</li>
