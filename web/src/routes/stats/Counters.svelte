<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Cancel01Icon } from '@hugeicons/core-free-icons';
	import { page } from '$app/state';
	import { href, percent, queryHref, type Card, type CardMap, type Schemas } from '#lib';
	import FilterMenu from '#lib/components/FilterMenu.svelte';
	import { buttonVariants } from '#lib/components/ui/button/index.js';
	import { toggle, type Filter } from '#lib/filter.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';

	let {
		data,
		against,
		cards,
		catalog
	}: { data: Schemas['Counters'] | null; against: string[]; cards: Card[]; catalog: CardMap } =
		$props();

	const MIN_GAMES = 10;
	const MAX = 4;
	const number = new Intl.NumberFormat(getLocale());
	const name = (id: string) => catalog.get(id)?.name ?? id.slice(0, 8);
	const pick: Filter = $derived({
		id: 'against',
		label: m.counters_pick(),
		multiple: true,
		options: cards
			.map((c) => ({ value: c.id, label: c.name ?? c.id }))
			.sort((a, b) => a.label.localeCompare(b.label, getLocale()))
	});
</script>

{#snippet avatar(id: string, size: string)}
	{@const card = catalog.get(id)}
	{#if card}
		<img
			src="/art/avatar/{id}.webp"
			alt=""
			loading="lazy"
			decoding="async"
			class="el-{card.element} {size} shrink-0 rounded-xs border-2 border-(--el) bg-(--el)"
		/>
	{/if}
{/snippet}

<section aria-labelledby="counters-title">
	<div class="grid gap-1 section-head">
		<h2 id="counters-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.counters_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.counters_text({ min: MIN_GAMES })}</p>
	</div>
	<div
		class="flex flex-wrap items-center gap-2 px-(--gutter) pt-2 pb-4"
		data-sveltekit-reset="false"
	>
		<FilterMenu filter={pick} selected={against.length ? against.join(',') : null} />
		{#each against as id (id)}
			<a
				href={queryHref(page.url, { against: toggle(against, id) })}
				class={buttonVariants({ variant: 'secondary', size: 'sm' })}
				aria-label={m.counters_remove({ name: name(id) })}
			>
				{@render avatar(id, 'size-5')}
				<span class={[!catalog.has(id) && 'font-mono text-xs']}>{name(id)}</span>
				<HugeiconsIcon icon={Cancel01Icon} aria-hidden="true" class="size-3.5" />
			</a>
		{/each}
		{#if against.length}
			<a
				href={queryHref(page.url, { against: null })}
				class={[buttonVariants({ variant: 'ghost', size: 'sm' }), 'text-muted-foreground']}
				>{m.clear()}</a
			>
		{/if}
		{#if against.length >= MAX}
			<span class="text-sm text-muted-foreground">{m.counters_max({ max: MAX })}</span>
		{/if}
	</div>
	{#if !data}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.counters_empty()}</p>
	{:else}
		<p class="px-(--gutter) pb-3 text-sm text-muted-foreground tabular-nums">
			{m.counters_games({ count: data.games })}
		</p>
		{#if data.cards.length}
			<div>
				<ul class="cells grid sm:grid-cols-2 lg:grid-cols-3">
					{#each data.cards as row (row.card)}
						{@const card = catalog.get(row.card)}
						<li class="flex items-center gap-3 px-(--gutter) py-3 text-sm">
							{@render avatar(row.card, 'size-8')}
							{#if card}
								<a href={href(`/cards/${row.card}`)} class="min-w-0 flex-1 truncate hover:underline"
									>{card.name ?? row.card}</a
								>
							{:else}
								<span class="min-w-0 flex-1 font-mono text-xs">{row.card.slice(0, 8)}</span>
							{/if}
							<span class="grid justify-items-end tabular-nums">
								<span><span class="sr-only">{m.win_rate()}</span> {percent(row.winRate)}</span>
								<span class="text-xs text-muted-foreground"
									>{number.format(row.games)} {m.games_unit({ count: row.games })}</span
								>
							</span>
						</li>
					{/each}
				</ul>
			</div>
		{:else}
			<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">
				{m.counters_none({ min: MIN_GAMES })}
			</p>
		{/if}
	{/if}
</section>
