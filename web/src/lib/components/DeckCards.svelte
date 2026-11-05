<script lang="ts">
	import type { Snippet } from 'svelte';
	import { percent, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import CardGrid from './CardGrid.svelte';
	import CardThumb from './CardThumb.svelte';
	import CardTile from './CardTile.svelte';
	import Change from './Change.svelte';

	let {
		cards,
		catalog,
		tiles = false,
		large = false,
		stats,
		below
	}: {
		cards: string[];
		catalog: ReadonlyMap<string, Schemas['CardBase']>;
		tiles?: boolean;
		large?: boolean;
		stats?: ReadonlyMap<string, Schemas['CardStats']>;
		below?: Snippet<[number]>;
	} = $props();
</script>

{#if tiles}
	<CardGrid columns={4}>
		{#each cards as id, i (id)}
			{@const card = catalog.get(id)}
			<li class="grid grid-cols-1 content-start">
				{#if card}
					<CardTile {card} />
				{:else}
					<div
						class="m-2 flex aspect-5/7 items-center justify-center border border-dashed p-3 text-center text-sm text-muted-foreground sm:m-3"
					>
						{m.unknown_card()}
						{id.slice(0, 8)}
					</div>
				{/if}
				{#if below}<div class="border-t p-3 sm:p-4">{@render below(i)}</div>{/if}
			</li>
		{/each}
	</CardGrid>
{:else}
	<ul class={['grid grid-cols-4 gap-2 pt-[2.2%] pl-[2.2%]', stats && 'lg:gap-x-5']}>
		{#each cards as id (id)}
			<li class={{ '@container': stats }}>
				<CardThumb {id} card={catalog.get(id)} {large} />
				{#if stats}
					{@const s = stats.get(id)}
					{@const games = s?.games ?? 0}
					{@const name = catalog.get(id)?.name}
					<div
						class="mt-3 grid gap-1 text-xs leading-4 wrap-anywhere text-muted-foreground tabular-nums"
					>
						{#if name}
							<span
								class="truncate text-sm/5 font-medium text-foreground @max-[4.25rem]:hidden"
								aria-hidden="true">{name}</span
							>
						{/if}
						<span class="flex flex-wrap items-baseline gap-x-2">
							<span class="text-sm/5 text-foreground"
								><span class="sr-only">{m.win_rate()}</span> {percent(s?.winRate)}</span
							>
							<Change
								value={s?.winRateChange}
								label={m.win_rate()}
								color
								class="@max-[7rem]:hidden"
							/>
						</span>
						<span class="eyebrow @max-[7rem]:text-[10px] @max-[4.25rem]:hidden"
							>{games} {m.games_unit({ count: games })}</span
						>
					</div>
				{/if}
			</li>
		{/each}
	</ul>
{/if}
