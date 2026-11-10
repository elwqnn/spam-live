<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowLeft02Icon } from '@hugeicons/core-free-icons';
	import { goto } from '$app/navigation';
	import { href } from '$host';
	import DeckForm from '#ui/components/DeckForm.svelte';
	import TitleRow from '#ui/components/TitleRow.svelte';
	import { m } from '$paraglide/messages.js';
	import type { Writes } from '#ui/writes.js';
	import type { editDeckData } from './data.js';

	let {
		data,
		writes
	}: { data: Awaited<ReturnType<typeof editDeckData>>; writes: Pick<Writes, 'updateDeck'> } =
		$props();
</script>

<svelte:head>
	<title>{m.edit_deck()} · {m.site_name()}</title>
</svelte:head>

<TitleRow class="grid gap-2 row">
	<a
		class="eyebrow wrap-anywhere text-muted-foreground transition-colors hover:text-foreground"
		href={href(`/decks/${data.deck.id}`)}
		><HugeiconsIcon
			icon={ArrowLeft02Icon}
			aria-hidden="true"
			class="inline size-[1em] align-[-0.125em]"
		/>
		{data.deck.name}</a
	>
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">{m.edit_deck()}</h1>
</TitleRow>
{#key data.deck.id}
	<DeckForm
		cards={data.cards}
		deck={data.deck}
		details={data.details}
		keywords={data.keywords}
		save={async (deck) => {
			await writes.updateDeck(data.deck.id, deck);
			await goto(href(`/decks/${data.deck.id}`), { invalidateAll: true });
		}}
	/>
{/key}
