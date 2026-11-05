<script lang="ts">
	import { href, percent, type Schemas } from '#lib';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import Author from './Author.svelte';
	import DeckCards from './DeckCards.svelte';

	let {
		deck,
		catalog
	}: { deck: Schemas['DeckItem']; catalog: ReadonlyMap<string, Schemas['CardBase']> } = $props();
</script>

<li
	class="flex flex-col gap-3 px-(--gutter) py-4 transition-colors [contain-intrinsic-size:auto_205px] [content-visibility:auto] hover:bg-row sm:py-5"
>
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
	<dl class="mt-auto flex flex-wrap gap-x-6 gap-y-2 text-[13px] tabular-nums">
		<div class="grid gap-0.5">
			<dt class="eyebrow text-[10px] text-muted-foreground">{m.votes()}</dt>
			<dd>{deck.votes}</dd>
		</div>
		<div class="grid gap-0.5">
			<dt class="eyebrow text-[10px] text-muted-foreground">{m.games()}</dt>
			<dd>{deck.games}</dd>
		</div>
		<div class="grid gap-0.5">
			<dt class="eyebrow text-[10px] text-muted-foreground">{m.win_rate()}</dt>
			<dd>{percent(deck.winRate)}</dd>
		</div>
	</dl>
</li>
