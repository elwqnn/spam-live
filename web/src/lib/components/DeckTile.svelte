<script lang="ts">
	import { href, percent, type Schemas } from '#lib';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { Reason } from '#lib/promote.js';
	import Author from './Author.svelte';
	import DeckCards from './DeckCards.svelte';
	import DeckComments from './DeckComments.svelte';
	import DeckVotes from './DeckVotes.svelte';
	import LiveLines from './LiveLines.svelte';
	import Time from './Time.svelte';

	let {
		deck,
		catalog,
		reasons = [],
		large = false
	}: {
		deck: Schemas['DeckItem'];
		catalog: ReadonlyMap<string, Schemas['CardBase']>;
		reasons?: Reason[];
		large?: boolean;
	} = $props();
</script>

<li
	class={[
		'flex min-w-0 flex-col gap-3 px-(--gutter) py-4 transition-colors [contain-intrinsic-size:auto_205px] [content-visibility:auto] sm:py-5',
		reasons.length ? 'relative isolate bg-card' : 'hover:bg-row',
		large && 'lg:col-span-2 lg:row-span-2 lg:[contain-intrinsic-size:auto_480px]'
	]}
>
	{#if reasons.length}
		<LiveLines line={1} speed={1 / 3} class="opacity-12 [:hover>&]:opacity-12" />
	{/if}
	<div class="flex items-start gap-3">
		<div class="flex min-w-0 flex-1 flex-wrap items-center gap-x-3 gap-y-1">
			<a
				class="truncate text-[15px] font-medium hover:underline"
				href={href(`/decks/${deck.id}`)}
				title={deck.name}>{deck.name}</a
			>
			{#if deck.status === 'draft'}<Badge variant="secondary">{m.draft()}</Badge>{/if}
			<span class="flex min-w-0 text-sm text-muted-foreground">
				<Author name={deck.authorName} avatar={deck.authorAvatar} />
			</span>
		</div>
		<DeckComments {deck} />
		<DeckVotes {deck} />
	</div>
	<DeckCards cards={deck.cards} {catalog} {large} />
	<dl class="mt-auto flex flex-wrap gap-x-6 gap-y-2 text-[13px] tabular-nums">
		<div class="grid gap-0.5">
			<dt class="eyebrow text-[10px] text-muted-foreground">{m.win_rate()}</dt>
			<dd class={{ 'lg:text-2xl': large, 'shader-text': reasons.includes('winrate') }}>
				{percent(deck.winRate)}
			</dd>
		</div>
		<div class="grid gap-0.5">
			<dt class="eyebrow text-[10px] text-muted-foreground">{m.games()}</dt>
			<dd class={{ 'lg:text-2xl': large }}>{deck.games}</dd>
		</div>
		<div class="grid gap-0.5">
			<dt class="eyebrow text-[10px] text-muted-foreground">{m.added()}</dt>
			<dd class="text-muted-foreground"><Time iso={deck.publishedAt ?? deck.createdAt} /></dd>
		</div>
	</dl>
</li>
