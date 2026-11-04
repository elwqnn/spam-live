<script lang="ts">
	import { href } from '#lib';
	import DeckForm from '#lib/components/DeckForm.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();
</script>

<svelte:head>
	<title>{m.edit_deck()} · {m.site_name()}</title>
</svelte:head>

<div class="grid gap-2 row">
	<a
		class="eyebrow wrap-anywhere text-muted-foreground transition-colors hover:text-foreground"
		href={href(`/decks/${data.deck.id}`)}><span aria-hidden="true">←</span> {data.deck.name}</a
	>
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">{m.edit_deck()}</h1>
</div>
{#key data.deck.id}
	<DeckForm cards={data.cards} deck={data.deck} details={data.details} error={form?.error} />
{/key}
