<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowLeft02Icon } from '@hugeicons/core-free-icons';
	import { goto } from '$app/navigation';
	import { href } from '$host';
	import DeckForm from '#ui/components/DeckForm.svelte';
	import TitleRow from '#ui/components/TitleRow.svelte';
	import { m } from '$paraglide/messages.js';
	import type { Writes } from '#ui/writes.js';
	import type { newDeckData } from './data.js';

	let {
		data,
		writes
	}: { data: Awaited<ReturnType<typeof newDeckData>>; writes: Pick<Writes, 'createDeck'> } =
		$props();
</script>

<svelte:head>
	<title>{m.new_deck()} · {m.site_name()}</title>
</svelte:head>

<TitleRow class="grid gap-2 row">
	<a
		class="eyebrow text-muted-foreground transition-colors hover:text-foreground"
		href={href('/decks')}
		><HugeiconsIcon
			icon={ArrowLeft02Icon}
			aria-hidden="true"
			class="inline size-[1em] align-[-0.125em]"
		/>
		{m.all_decks()}</a
	>
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">{m.new_deck()}</h1>
	<p class="text-sm text-muted-foreground">{m.new_deck_text()}</p>
</TitleRow>
<DeckForm
	cards={data.cards}
	keywords={data.keywords}
	save={async (deck) => {
		const id = await writes.createDeck(deck);
		await goto(href(`/decks/${id}`), { invalidateAll: true });
	}}
/>
