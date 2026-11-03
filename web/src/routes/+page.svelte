<script lang="ts">
	import { href, percent } from '#lib';
	import DeckCards from '#lib/components/DeckCards.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import * as Card from '#lib/components/ui/card/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const catalog = $derived(new Map(data.cards.map((c) => [c.id, c])));
</script>

<svelte:head>
	<meta name="description" content={m.site_tagline()} />
</svelte:head>

<div class="grid gap-6">
	<section class="grid gap-2">
		<h1 class="font-heading text-2xl font-semibold">{m.site_name()}</h1>
		<p class="text-muted-foreground">{m.site_tagline()}</p>
	</section>

	<div class="grid gap-6 md:grid-cols-2">
		<Card.Root>
			<Card.Header>
				<Card.Title>{m.home_top_decks()}</Card.Title>
				<Card.Action>
					<a class="text-sm hover:underline" href={href('/stats')}>{m.see_all_stats()}</a>
				</Card.Action>
			</Card.Header>
			<Card.Content>
				{#if data.top.length}
					<ol class="grid gap-3">
						{#each data.top as deck (deck.cards.join())}
							<li class="grid gap-1">
								<DeckCards cards={deck.cards} {catalog} />
								<p class="text-xs text-muted-foreground">
									{m.win_rate()}: {percent(deck.winRate)} · {m.games()}: {deck.games}
								</p>
							</li>
						{/each}
					</ol>
				{:else}
					<p class="text-sm text-muted-foreground">{m.home_no_top_decks()}</p>
				{/if}
			</Card.Content>
		</Card.Root>

		<Card.Root>
			<Card.Header>
				<Card.Title>{m.home_recent_decks()}</Card.Title>
			</Card.Header>
			<Card.Content>
				{#if data.recent.length}
					<ul class="grid gap-3">
						{#each data.recent as deck (deck.id)}
							<li class="grid gap-1">
								<p class="text-sm">
									<a class="font-medium hover:underline" href={href(`/decks/${deck.id}`)}
										>{deck.name}</a
									>
									<span class="text-muted-foreground">{m.by_author({ name: deck.authorName })}</span
									>
								</p>
								<DeckCards cards={deck.cards} {catalog} />
							</li>
						{/each}
					</ul>
				{:else}
					<p class="text-sm text-muted-foreground">{m.home_no_recent_decks()}</p>
				{/if}
			</Card.Content>
		</Card.Root>
	</div>

	<Card.Root>
		<Card.Header>
			<Card.Title>{m.import_title()}</Card.Title>
			<Card.Description>{m.import_text()}</Card.Description>
		</Card.Header>
		<Card.Footer>
			<Button href={href('/import')}>{m.import_title()}</Button>
		</Card.Footer>
	</Card.Root>
</div>
