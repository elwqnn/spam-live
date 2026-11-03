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

<div class="grid gap-4">
	<div class="grid gap-2">
		<a class="text-sm text-muted-foreground hover:underline" href={href('/decks')}
			>{m.all_decks()}</a
		>
		<h1 class="font-heading text-2xl font-semibold">{m.new_deck()}</h1>
		<p class="text-sm text-muted-foreground">{m.new_deck_text()}</p>
	</div>
	<DeckForm cards={data.cards} error={form?.error} />
</div>
