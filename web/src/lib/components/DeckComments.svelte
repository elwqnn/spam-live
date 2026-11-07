<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Comment01Icon } from '@hugeicons/core-free-icons';
	import { onMount } from 'svelte';
	import { href, type Schemas } from '#lib';
	import { comments, live } from '#lib/live.js';
	import { m } from '#lib/paraglide/messages.js';

	let { deck }: { deck: Pick<Schemas['DeckItem'], 'id' | 'comments'> } = $props();

	const count = $derived(comments.get(deck.id)?.comments ?? deck.comments);

	onMount(() => live());
</script>

<a
	class="flex shrink-0 items-center gap-1 text-sm whitespace-nowrap text-muted-foreground tabular-nums transition-colors hover:text-foreground"
	href={href(`/decks/${deck.id}#comments`)}
	aria-label={m.comments_count({ count })}
>
	<HugeiconsIcon icon={Comment01Icon} aria-hidden="true" class="size-4" />
	{count}
</a>
