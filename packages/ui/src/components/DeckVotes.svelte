<script lang="ts">
	import { page } from '$app/state';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowDownBigIcon, ArrowUpBigIcon } from '@hugeicons/core-free-icons';
	import { onMount } from 'svelte';
	import type { Schemas } from '#ui';
	import { loginHref, type PageData } from '$host';
	import { live, votes } from '#ui/live.js';
	import { hydrated } from '#ui/ready.svelte.js';
	import type { Writes } from '#ui/writes.js';
	import { m } from '$paraglide/messages.js';

	let {
		deck,
		vote
	}: {
		deck: Pick<
			Schemas['DeckItem'],
			'id' | 'status' | 'authorId' | 'upvotes' | 'downvotes' | 'myVote'
		>;
		vote: Writes['vote'];
	} = $props();

	const SIDES = [1, -1] as const;
	const row = 'flex shrink-0 items-center gap-2 text-sm text-muted-foreground';
	const control = 'flex items-center gap-1 transition-colors hover:text-foreground';

	const me = $derived((page.data as PageData).me);
	const open = $derived(deck.status === 'public');
	let mine = $derived(deck.myVote ?? 0);
	let up = $derived(deck.upvotes);
	let down = $derived(deck.downvotes);
	let pending = false;
	let server: unknown;
	let pop = $state(0);
	const js = hydrated();

	onMount(() => live());

	$effect(() => {
		const counts = votes.get(deck.id);
		server = [deck.upvotes, deck.downvotes, counts];
		if (!counts) return;
		up = counts.upvotes;
		down = counts.downvotes;
	});

	const move = (side: number, by: number) => {
		if (side === 1) up += by;
		if (side === -1) down += by;
	};

	async function cast(next: 1 | -1 | 0) {
		if (pending) return;
		pending = true;
		const was = { mine, up, down, server };
		move(mine, -1);
		move(next, 1);
		mine = next;
		pop = next;
		try {
			await vote(deck.id, next);
			pending = false;
		} catch {
			pending = false;
			mine = was.mine;
			pop = 0;
			if (server !== was.server) return;
			up = was.up;
			down = was.down;
		}
	}
</script>

{#snippet arrow(side: 1 | -1, action: boolean)}
	{@const chosen = mine === side}
	<span
		class={['flex', chosen && pop === side && 'motion-safe:animate-pop']}
		onanimationend={() => (pop = 0)}
	>
		<HugeiconsIcon
			icon={side === 1 ? ArrowUpBigIcon : ArrowDownBigIcon}
			aria-hidden="true"
			class={chosen ? `size-4 fill-current ${side === 1 ? 'text-terra' : 'text-slate'}` : 'size-4'}
		/>
	</span>
	<span class="sr-only"
		>{action
			? side === 1
				? m.upvote()
				: m.downvote()
			: side === 1
				? m.upvotes()
				: m.downvotes()}</span
	>
	<span class={['tabular-nums', chosen && 'text-foreground']}>{side === 1 ? up : down}</span>
{/snippet}

{#if !me && open}
	<div class={row}>
		{#each SIDES as side (side)}
			<a class={control} href={loginHref(page.url)} rel="external">{@render arrow(side, true)}</a>
		{/each}
	</div>
{:else if me && open && deck.authorId !== me.id}
	<div class={row}>
		{#each SIDES as side (side)}
			<button
				class={[control, 'cursor-pointer']}
				type="button"
				{...js.lock()}
				aria-pressed={mine === side}
				onclick={() => cast(mine === side ? 0 : side)}>{@render arrow(side, true)}</button
			>
		{/each}
	</div>
{:else}
	<div class={row}>
		{#each SIDES as side (side)}
			<span class="flex items-center gap-1">{@render arrow(side, false)}</span>
		{/each}
	</div>
{/if}
