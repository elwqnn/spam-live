<script lang="ts">
	import { enhance, type SubmitFunction } from '$app/forms';
	import { page } from '$app/state';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowDownBigIcon, ArrowUpBigIcon } from '@hugeicons/core-free-icons';
	import { href, loginHref, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';

	let {
		deck
	}: {
		deck: Pick<
			Schemas['DeckItem'],
			'id' | 'status' | 'authorId' | 'upvotes' | 'downvotes' | 'myVote'
		>;
	} = $props();

	const SIDES = [1, -1] as const;
	const row = 'flex shrink-0 items-center gap-2 text-sm text-muted-foreground';
	const control = 'flex items-center gap-1 transition-colors hover:text-foreground';

	const me: Schemas['Me'] | null = $derived(page.data.me);
	const open = $derived(deck.status === 'public');
	let mine = $derived(deck.myVote ?? 0);
	let up = $derived(deck.upvotes);
	let down = $derived(deck.downvotes);
	let pending = false;
	let pop = $state(0);

	const move = (side: number, by: number) => {
		if (side === 1) up += by;
		if (side === -1) down += by;
	};

	const vote: SubmitFunction = ({ cancel, submitter }) => {
		if (pending) return cancel();
		pending = true;
		const next = Number((submitter as HTMLButtonElement | null)?.value);
		const was = { mine, up, down };
		move(mine, -1);
		move(next, 1);
		mine = next;
		pop = next;
		return ({ result }) => {
			pending = false;
			if (result.type !== 'success') {
				mine = was.mine;
				up = was.up;
				down = was.down;
				pop = 0;
			}
		};
	};
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
	<form class={row} method="POST" action={`${href(`/decks/${deck.id}`)}?/vote`} use:enhance={vote}>
		{#each SIDES as side (side)}
			<button
				class={[control, 'cursor-pointer']}
				type="submit"
				name="vote"
				value={mine === side ? '0' : String(side)}
				aria-pressed={mine === side}>{@render arrow(side, true)}</button
			>
		{/each}
	</form>
{:else}
	<div class={row}>
		{#each SIDES as side (side)}
			<span class="flex items-center gap-1">{@render arrow(side, false)}</span>
		{/each}
	</div>
{/if}
