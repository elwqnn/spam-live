<script lang="ts">
	import { enhance, type SubmitFunction } from '$app/forms';
	import { page } from '$app/state';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { FavouriteIcon } from '@hugeicons/core-free-icons';
	import { href, loginHref, percent, type Schemas } from '#lib';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { Reason } from '#lib/promote.js';
	import Author from './Author.svelte';
	import DeckCards from './DeckCards.svelte';
	import LiveLines from './LiveLines.svelte';
	import Time from './Time.svelte';

	let {
		deck,
		catalog,
		reasons = [],
		large = false
	}: {
		deck: Schemas['DeckItem'];
		catalog: ReadonlyMap<string, Schemas['CardBase']>;
		reasons?: Reason[];
		large?: boolean;
	} = $props();

	const me: Schemas['Me'] | null = $derived(page.data.me);
	const open = $derived(deck.status === 'public');
	let voted = $derived(!!deck.myVote);
	let votes = $derived(deck.votes);
	let pending = false;
	let pop = $state(false);

	const vote: SubmitFunction = ({ cancel }) => {
		if (pending) return cancel();
		pending = true;
		const [was, count] = [voted, votes];
		voted = !was;
		votes = count + (was ? -1 : 1);
		pop = !was;
		return ({ result }) => {
			pending = false;
			if (result.type !== 'success') {
				voted = was;
				votes = count;
				pop = false;
			}
		};
	};
</script>

{#snippet heart(label: string)}
	<span
		class={['flex', pop && voted && 'motion-safe:animate-pop']}
		onanimationend={() => (pop = false)}
	>
		<HugeiconsIcon
			icon={FavouriteIcon}
			aria-hidden="true"
			class={voted ? 'size-4 fill-current text-terra' : 'size-4'}
		/>
	</span>
	<span class="sr-only">{label}</span>
	<span class="tabular-nums">{votes}</span>
{/snippet}

<li
	class={[
		'flex flex-col gap-3 px-(--gutter) py-4 transition-colors [contain-intrinsic-size:auto_205px] [content-visibility:auto] sm:py-5',
		reasons.length ? 'relative isolate bg-card' : 'hover:bg-row',
		large && 'lg:col-span-2 lg:row-span-2 lg:[contain-intrinsic-size:auto_480px]'
	]}
>
	{#if reasons.length}
		<LiveLines line={1} speed={1 / 3} class="opacity-12 [:hover>&]:opacity-12" />
	{/if}
	<div class="flex items-start gap-3">
		<div class="flex min-w-0 flex-1 flex-wrap items-center gap-x-3 gap-y-1">
			<a
				class="text-[15px] font-medium wrap-anywhere hover:underline"
				href={href(`/decks/${deck.id}`)}>{deck.name}</a
			>
			{#if deck.status === 'draft'}<Badge variant="secondary">{m.draft()}</Badge>{/if}
			<span class="flex min-w-0 text-sm text-muted-foreground">
				<Author name={deck.authorName} avatar={deck.authorAvatar} />
			</span>
		</div>
		{#if !me && open}
			<a
				class="flex shrink-0 items-center gap-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground"
				href={loginHref(page.url)}
				rel="external">{@render heart(m.vote())}</a
			>
		{:else if me && open && deck.authorId !== me.id}
			<form
				class="shrink-0"
				method="POST"
				action={`${href(`/decks/${deck.id}`)}?/vote`}
				use:enhance={vote}
			>
				<input type="hidden" name="vote" value={voted ? '0' : '1'} />
				<button
					type="submit"
					aria-pressed={voted}
					class={[
						'flex cursor-pointer items-center gap-1.5 text-sm transition-colors',
						voted ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'
					]}>{@render heart(m.vote())}</button
				>
			</form>
		{:else}
			<span class="flex shrink-0 items-center gap-1.5 text-sm text-muted-foreground"
				>{@render heart(m.votes())}</span
			>
		{/if}
	</div>
	<DeckCards cards={deck.cards} {catalog} {large} />
	<dl class="mt-auto flex flex-wrap gap-x-6 gap-y-2 text-[13px] tabular-nums">
		<div class="grid gap-0.5">
			<dt class="eyebrow text-[10px] text-muted-foreground">{m.win_rate()}</dt>
			<dd class={{ 'lg:text-2xl': large, 'shader-text': reasons.includes('winrate') }}>
				{percent(deck.winRate)}
			</dd>
		</div>
		<div class="grid gap-0.5">
			<dt class="eyebrow text-[10px] text-muted-foreground">{m.games()}</dt>
			<dd class={{ 'lg:text-2xl': large }}>{deck.games}</dd>
		</div>
		<div class="grid gap-0.5">
			<dt class="eyebrow text-[10px] text-muted-foreground">{m.added()}</dt>
			<dd class="text-muted-foreground"><Time iso={deck.publishedAt ?? deck.createdAt} /></dd>
		</div>
	</dl>
</li>
