<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Comment01Icon } from '@hugeicons/core-free-icons';
	import { onMount } from 'svelte';
	import { formatDate, type Schemas } from '#ui';
	import { client, href } from '$host';
	import { comments, live } from '#ui/live.js';
	import { m } from '$paraglide/messages.js';
	import Author from './Author.svelte';
	import Tooltip from './Tooltip.svelte';

	let { deck }: { deck: Pick<Schemas['DeckItem'], 'id' | 'comments'> } = $props();

	const count = $derived(comments.get(deck.id)?.comments ?? deck.comments);
	let latest = $state<Schemas['Comment'][]>();
	let loaded = -1;

	async function load() {
		if (!count || loaded === count) return;
		loaded = count;
		const data = await client()
			.GET('/api/decks/{id}/comments', { params: { path: { id: deck.id } } })
			.then(
				(r) => r.data,
				() => undefined
			);
		if (data) latest = data.slice(-3).reverse();
		else loaded = -1;
	}

	onMount(() => live());
</script>

{#snippet preview()}
	{#if latest}
		<ul class="divide-y">
			{#each latest as c (c.id)}
				<li class="grid gap-1 px-3 py-2">
					<div class="flex items-center gap-2 text-xs">
						<Author name={c.authorName} avatar={c.authorAvatar} />
						<time datetime={c.createdAt} class="ml-auto shrink-0 text-muted-foreground"
							>{formatDate(c.createdAt)}</time
						>
					</div>
					<p class="line-clamp-3 leading-5 wrap-anywhere whitespace-pre-line" data-private>
						{c.body}
					</p>
				</li>
			{/each}
		</ul>
	{:else}
		<p class="px-3 py-2 text-muted-foreground">{m.loading()}</p>
	{/if}
{/snippet}

<Tooltip content={count ? preview : undefined} class="w-80 max-w-80 p-0 text-sm">
	{#snippet children(tip)}
		<a
			{...tip}
			onpointerenter={(e) => {
				load();
				tip.onpointerenter?.(e);
			}}
			onfocus={(e) => {
				load();
				tip.onfocus?.(e);
			}}
			class="flex shrink-0 items-center gap-1 text-sm whitespace-nowrap text-muted-foreground tabular-nums transition-colors hover:text-foreground"
			href={href(`/decks/${deck.id}#comments`)}
			aria-label={m.comments_count({ count })}
		>
			<HugeiconsIcon icon={Comment01Icon} aria-hidden="true" class="size-4" />
			{count}
		</a>
	{/snippet}
</Tooltip>
