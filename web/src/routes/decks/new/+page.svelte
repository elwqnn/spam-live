<script lang="ts">
	import { href } from '#lib';
	import DeckForm from '#lib/components/DeckForm.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();
</script>

<svelte:head>
	<title>{m.new_deck()} · {m.site_name()}</title>
</svelte:head>

<div class="grid gap-6">
	<div class="grid gap-2">
		<a class="mono-label text-muted-foreground hover:text-foreground" href={href('/decks')}
			><span aria-hidden="true">←</span> {m.all_decks()}</a
		>
		<h1 class="font-heading text-3xl font-medium tracking-tight sm:text-4xl">{m.new_deck()}</h1>
		<p class="text-sm text-muted-foreground">{m.new_deck_text()}</p>
	</div>
	<DeckForm cards={data.cards} error={form?.error} />
</div>
