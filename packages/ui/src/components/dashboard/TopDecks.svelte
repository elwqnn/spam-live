<script lang="ts">
	import { percent, winRateColor, type CardMap, type Schemas } from '#ui';
	import { href } from '$host';
	import { buttonVariants } from '#ui/components/ui/button/index.js';
	import type { Writes } from '#ui/writes.js';
	import { m } from '$paraglide/messages.js';
	import CardAvatar from './CardAvatar.svelte';
	import PublishDialog from './PublishDialog.svelte';

	let { top, catalog, writes }: { top: Schemas['TopDeck'][]; catalog: CardMap; writes: Writes } =
		$props();
</script>

<section aria-labelledby="top-decks-title">
	<div class="grid gap-1 section-head">
		<h2 id="top-decks-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.dashboard_top_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.dashboard_top_text()}</p>
	</div>
	{#if top.length}
		<ol class="divide-y">
			{#each top as t (t.cards.join())}
				{@const deck = t.deck}
				<li class="flex flex-wrap items-center gap-x-5 gap-y-3 px-(--gutter) py-4">
					<ul class="flex gap-1.5">
						{#each deck?.cards ?? t.cards as id (id)}
							<li><CardAvatar {id} card={catalog.get(id)} class="size-12" /></li>
						{/each}
					</ul>
					<div class="grid min-w-0 flex-1 gap-1">
						<p class="truncate font-medium" data-private>
							{deck?.name ?? m.dashboard_deck_played()}
						</p>
						<p class="flex flex-wrap gap-x-3 text-sm text-muted-foreground tabular-nums">
							<span>{t.games} {m.games_unit({ count: t.games })}</span>
							<span style:color={winRateColor(t.winRate)}
								><span class="sr-only">{m.win_rate()}</span> {percent(t.winRate)}</span
							>
						</p>
					</div>
					<div class="grid justify-items-end gap-1">
						{#if deck?.status === 'public'}
							<a
								class={buttonVariants({ variant: 'ghost', size: 'sm' })}
								href={href(`/decks/${deck.id}`)}>{m.dashboard_published()}</a
							>
						{:else if deck}
							<PublishDialog {deck} {writes} />
						{:else}
							<p class="max-w-48 text-right text-xs text-muted-foreground">
								{m.dashboard_deck_unsaved()}
							</p>
						{/if}
					</div>
				</li>
			{/each}
		</ol>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.dashboard_top_empty()}</p>
	{/if}
</section>
