<script lang="ts">
	import { href } from '#lib';
	import DeckForm from '#lib/components/DeckForm.svelte';
	import TitleRow from '#lib/components/TitleRow.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();
</script>

<svelte:head>
	<title>{m.new_deck()} · {m.site_name()}</title>
</svelte:head>

<TitleRow class="grid gap-2 row">
	<a
		class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
		href={href('/decks')}><span aria-hidden="true">←</span> {m.all_decks()}</a
	>
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">{m.new_deck()}</h1>
	<p class="text-sm text-muted-foreground">{m.new_deck_text()}</p>
</TitleRow>
<DeckForm cards={data.cards} keywords={data.keywords} error={form?.error} />
