<script lang="ts">
	import { href, percent, type CardMap, type Schemas } from '#lib';
	import { cardPreview } from '#lib/components/CardPreview.svelte';
	import { m } from '#lib/paraglide/messages.js';

	let { data, catalog }: { data: Schemas['MatchupStats']; catalog: CardMap } = $props();

	const MIN_GAMES = 10;
	const cells = $derived(new Map(data.cells.map((c) => [`${c.card} ${c.opponent}`, c])));
	const tint = (rate: number) =>
		rate === 0.5
			? undefined
			: `color-mix(in oklab, var(${rate > 0.5 ? '--up' : '--down'}) ${Math.min(Math.abs(rate - 0.5) / 0.2, 1) * 35}%, transparent)`;
</script>

{#snippet avatar(id: string, element: string)}
	<img
		src="/art/avatar/{id}.webp"
		alt=""
		loading="lazy"
		decoding="async"
		class="el-{element} size-7 shrink-0 rounded-xs border-2 border-(--el) bg-(--el) transition-colors group-hover:border-foreground"
	/>
{/snippet}

<section aria-labelledby="matchups-title">
	<div class="grid gap-1 section-head">
		<h2 id="matchups-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.matchups_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.matchups_text({ min: MIN_GAMES })}</p>
	</div>
	{#if data.cards.length}
		<p class="flex flex-wrap gap-x-4 gap-y-1 px-(--gutter) pt-2 pb-3 eyebrow text-muted-foreground">
			<span>{m.matchups_rows()}</span>
			<span>{m.matchups_columns()}</span>
		</p>
		<div class="overflow-x-auto pb-4">
			<table class="w-full border-collapse text-sm tabular-nums">
				<caption class="sr-only">{m.matchups_caption()}</caption>
				<thead>
					<tr>
						<td class="sticky left-0 z-1 bg-background"></td>
						{#each data.cards as id (id)}
							{@const card = catalog.get(id)}
							<th scope="col" class="px-1 pb-2 font-normal last:pr-(--gutter)">
								{#if card}
									<a
										href={href(`/cards/${id}`)}
										class="group mx-auto block w-fit rounded-xs"
										{@attach cardPreview(card)}
									>
										{@render avatar(id, card.element)}
										<span class="sr-only">{card.name ?? id}</span>
									</a>
								{:else}
									<span class="font-mono text-xs">{id.slice(0, 8)}</span>
								{/if}
							</th>
						{/each}
					</tr>
				</thead>
				<tbody>
					{#each data.cards as id (id)}
						{@const card = catalog.get(id)}
						<tr class="border-t">
							<th
								scope="row"
								class="sticky left-0 z-1 bg-background py-1 pr-3 pl-(--gutter) text-left font-normal"
							>
								{#if card}
									<span class="flex items-center gap-2">
										{@render avatar(id, card.element)}
										<a
											href={href(`/cards/${id}`)}
											class="max-w-28 truncate hover:underline sm:max-w-44">{card.name ?? id}</a
										>
									</span>
								{:else}
									<span class="font-mono text-xs">{id.slice(0, 8)}</span>
								{/if}
							</th>
							{#each data.cards as opponent (opponent)}
								{@const cell = id === opponent ? undefined : cells.get(`${id} ${opponent}`)}
								{#if cell?.winRate != null && cell.games >= MIN_GAMES}
									<td
										class="min-w-14 px-1 py-1 text-center last:pr-(--gutter)"
										style:background-color={tint(cell.winRate)}
									>
										<span aria-hidden="true">{percent(cell.winRate)}</span>
										<span class="sr-only"
											>{m.matchup_cell({ rate: percent(cell.winRate), count: cell.games })}</span
										>
									</td>
								{:else}
									<td
										class="min-w-14 px-1 py-1 text-center text-muted-foreground last:pr-(--gutter)"
									>
										<span aria-hidden="true">—</span>
										{#if cell}<span class="sr-only">{m.matchup_few({ min: MIN_GAMES })}</span>{/if}
									</td>
								{/if}
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.no_stats()}</p>
	{/if}
</section>
