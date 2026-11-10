<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Add01Icon, Cancel01Icon, Search01Icon } from '@hugeicons/core-free-icons';
	import { page } from '$app/state';
	import { fold, percent, winRateColor, type Card, type CardMap, type Schemas } from '#ui';
	import { art, href, queryHref } from '$host';
	import CardThumb from '#ui/components/CardThumb.svelte';
	import { Button, buttonVariants } from '#ui/components/ui/button/index.js';
	import { toggle } from '#ui/filter.js';
	import { m } from '$paraglide/messages.js';
	import { getLocale } from '$paraglide/runtime.js';

	let {
		data,
		against,
		cards,
		catalog
	}: { data: Schemas['Counters'] | null; against: string[]; cards: Card[]; catalog: CardMap } =
		$props();

	const MAX = 4;
	const number = new Intl.NumberFormat(getLocale());
	const name = (id: string) => catalog.get(id)?.name ?? id.slice(0, 8);

	let dialog = $state<HTMLDialogElement>();
	let query = $state('');
	const options = $derived(
		cards
			.filter((c) => !against.includes(c.id))
			.map((c) => ({ card: c, label: c.name ?? c.id, key: fold(c.name ?? c.id) }))
			.sort((a, b) => a.label.localeCompare(b.label, getLocale()))
	);
	const shown = $derived(options.filter((o) => o.key.includes(fold(query.trim()))));
</script>

{#snippet avatar(id: string, size: string)}
	{@const card = catalog.get(id)}
	{#if card}
		<img
			src={art(`avatar/${id}.webp`)}
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
		<p class="text-sm text-muted-foreground">{m.counters_text()}</p>
	</div>
	<div class="grid gap-3 px-(--gutter) pt-2 pb-5" data-sveltekit-reset="false">
		<ul class="grid max-w-md grid-cols-4 gap-2 pt-[2.2%] pl-[2.2%] sm:gap-3">
			{#each against as id (id)}
				<li class="relative">
					<CardThumb {id} card={catalog.get(id)} />
					<a
						href={queryHref(page.url, { against: toggle(against, id) })}
						aria-label={m.counters_remove({ name: name(id) })}
						class="absolute -top-2 -right-2 z-1 grid size-6 place-items-center rounded-xs border bg-popover text-muted-foreground transition-colors hover:border-foreground/40 hover:text-foreground focus-visible:outline-2 focus-visible:outline-offset-2"
					>
						<HugeiconsIcon icon={Cancel01Icon} aria-hidden="true" class="size-3.5" />
					</a>
				</li>
			{/each}
			{#each Array.from({ length: MAX - against.length }, (_, i) => i) as slot (slot)}
				<li>
					<button
						type="button"
						commandfor="counters-picker"
						command="show-modal"
						aria-label={m.counters_add()}
						class="grid aspect-[1432/2048] w-full place-items-center rounded-xs border border-dashed border-foreground/25 text-muted-foreground transition-colors hover:border-foreground/60 hover:text-foreground focus-visible:border-foreground/60 focus-visible:text-foreground focus-visible:outline-2 focus-visible:outline-offset-4"
					>
						<HugeiconsIcon icon={Add01Icon} aria-hidden="true" class="size-6" />
					</button>
				</li>
			{/each}
		</ul>
		{#if against.length}
			<a
				href={queryHref(page.url, { against: null })}
				class={[buttonVariants({ variant: 'ghost', size: 'sm' }), 'w-fit text-muted-foreground']}
				>{m.clear()}</a
			>
		{/if}
	</div>
	{#if against.length < MAX}
		<dialog
			bind:this={dialog}
			id="counters-picker"
			aria-labelledby="counters-picker-title"
			closedby="any"
			onclose={() => (query = '')}
			class="mx-auto mt-0 mb-auto max-h-dvh w-full max-w-full flex-col border-b bg-popover p-0 text-popover-foreground backdrop:bg-black/60 open:flex sm:mt-[12vh] sm:max-h-[min(40rem,76vh)] sm:max-w-2xl sm:border"
		>
			<h2
				id="counters-picker-title"
				class="px-4 pt-4 pb-2 font-heading text-2xl font-normal tracking-tight"
			>
				{m.counters_picker()}
			</h2>
			<div class="flex items-center gap-3 border-b pr-2 pl-4">
				<HugeiconsIcon
					icon={Search01Icon}
					aria-hidden="true"
					class="size-4 shrink-0 text-muted-foreground"
				/>
				<input
					type="search"
					bind:value={query}
					aria-label={m.counters_search()}
					placeholder={m.counters_search()}
					class="h-12 min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground [&::-webkit-search-cancel-button]:hidden"
				/>
				<Button
					variant="ghost"
					size="icon-sm"
					commandfor="counters-picker"
					command="close"
					aria-label={m.close()}
				>
					<HugeiconsIcon icon={Cancel01Icon} aria-hidden="true" class="size-4" />
				</Button>
			</div>
			<div class="min-h-0 flex-1 overflow-y-auto p-2" data-sveltekit-reset="false">
				{#if shown.length}
					<ul class="grid grid-cols-2 gap-1 sm:grid-cols-3">
						{#each shown as option (option.card.id)}
							<li>
								<a
									href={queryHref(page.url, { against: toggle(against, option.card.id) })}
									onclick={() => dialog?.close()}
									class="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm transition-colors hover:bg-muted focus-visible:bg-muted focus-visible:outline-none"
								>
									{@render avatar(option.card.id, 'size-7')}
									<span class="min-w-0 truncate">{option.label}</span>
								</a>
							</li>
						{/each}
					</ul>
				{:else}
					<p class="px-2 py-6 text-center text-sm text-muted-foreground">
						{m.counters_no_match()}
					</p>
				{/if}
			</div>
		</dialog>
	{/if}
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
								<span style:color={winRateColor(row.winRate)}
									><span class="sr-only">{m.win_rate()}</span> {percent(row.winRate)}</span
								>
								<span class="text-xs text-muted-foreground"
									>{number.format(row.games)} {m.games_unit({ count: row.games })}</span
								>
							</span>
						</li>
					{/each}
				</ul>
			</div>
		{:else}
			<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.counters_none()}</p>
		{/if}
	{/if}
</section>
