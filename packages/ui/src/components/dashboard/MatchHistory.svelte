<script lang="ts">
	import { page } from '$app/state';
	import { modeName, type CardMap, type Schemas } from '#ui';
	import { queryHref } from '$host';
	import Time from '#ui/components/Time.svelte';
	import { Badge } from '#ui/components/ui/badge/index.js';
	import { buttonVariants } from '#ui/components/ui/button/index.js';
	import { m } from '$paraglide/messages.js';
	import CardAvatar from './CardAvatar.svelte';

	let {
		matches,
		catalog,
		next,
		first
	}: {
		matches: Schemas['MyMatch'][];
		catalog: CardMap;
		next: string | null;
		first: boolean;
	} = $props();

	const results = {
		win: [m.result_win, 'default'],
		loss: [m.result_loss, 'secondary'],
		draw: [m.result_draw, 'outline']
	} as const;
	const link = buttonVariants({ variant: 'ghost', size: 'sm' });

	function duration(ms: number) {
		const s = Math.round(ms / 1000);
		return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
	}
</script>

{#snippet cards(ids: string[], label: string)}
	<ul class="flex gap-1" aria-label={label}>
		{#each ids as id (id)}
			<li><CardAvatar {id} card={catalog.get(id)} class="size-7" /></li>
		{/each}
	</ul>
{/snippet}

<section aria-labelledby="history-title">
	<h2 id="history-title" class="section-head font-heading text-2xl font-normal tracking-tight">
		{m.me_matches()}
	</h2>
	{#if matches.length}
		<ul class="divide-y">
			{#each matches as match (match.gameUid)}
				{@const [label, variant] = results[match.result]}
				{@const time = duration(match.durationMs)}
				<li
					class="flex flex-wrap items-center gap-x-4 gap-y-2 px-(--gutter) py-2.5 text-sm transition-colors hover:bg-row"
				>
					<Badge {variant} class="w-18">{label()}</Badge>
					<span class="w-20 truncate font-medium">{modeName(match.gameMode)}</span>
					{@render cards(match.myCards, m.my_cards())}
					{@render cards(match.opponentCards, m.opponent_cards())}
					{#if match.isAi}<Badge variant="outline">{m.ai()}</Badge>{/if}
					{#if match.disputed}<Badge variant="destructive">{m.disputed()}</Badge>{/if}
					<span class="ml-auto flex gap-x-4 text-xs text-muted-foreground tabular-nums">
						<Time iso={match.startUtc} />
						<span
							><span class="sr-only">{m.duration({ time })}</span><span aria-hidden="true"
								>{time}</span
							></span
						>
					</span>
				</li>
			{/each}
		</ul>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.me_no_matches()}</p>
	{/if}
	{#if !first || next}
		<nav class="flex justify-between gap-2 border-t px-(--gutter) py-3" aria-label={m.pages()}>
			{#if !first}
				<a class={link} href={queryHref(page.url, { before: null })}>{m.first_page()}</a>
			{:else}
				<span></span>
			{/if}
			{#if next}
				<a class={link} href={queryHref(page.url, { before: next })}>{m.next_page()}</a>
			{/if}
		</nav>
	{/if}
</section>
