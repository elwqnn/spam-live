<script lang="ts">
	import { BarChart } from 'layerchart';
	import { href, percent, type CardMap, type Schemas } from '#lib';
	import * as Chart from '#lib/components/ui/chart/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';

	let { data, catalog }: { data: Schemas['PaceStats']; catalog: CardMap } = $props();

	type Pace = Schemas['CardPace'];

	const MIN_GAMES = 10;
	const number = new Intl.NumberFormat(getLocale());

	const rows = $derived.by(() => {
		if (!data.turns.length) return [];
		const counts = new Map(data.turns.map((t) => [t.turns, t.matches]));
		const first = data.turns[0].turns;
		const last = data.turns[data.turns.length - 1].turns;
		return Array.from({ length: last - first + 1 }, (_, i) => ({
			turns: first + i,
			matches: counts.get(first + i) ?? 0
		}));
	});
	let width = $state(0);
	const step = $derived(Math.ceil(rows.length / Math.max(2, Math.floor((width - 64) / 32))));
	const series = $derived([{ key: 'matches', label: m.pace_matches(), color: 'var(--straw)' }]);
	const config = $derived({ matches: { label: m.pace_matches(), color: 'var(--straw)' } });
	const yTicks = (scale: unknown) =>
		((scale as { ticks?: (n: number) => number[] }).ticks?.(4) ?? []).filter(Number.isInteger);
	const chartProps = $derived({
		xAxis: {
			ticks: () => rows.map((r) => r.turns).filter((t) => t % step === 0),
			format: (v: number) => number.format(v),
			label: m.pace_turns(),
			labelPlacement: 'end' as const
		},
		yAxis: { ticks: yTicks, format: (v: number) => number.format(v) },
		bars: { radius: 0, strokeWidth: 0 }
	});
	const padding = { top: 8, right: 8, bottom: 36, left: 48 };

	const diff = (c: Pace) => (c.shortWinRate ?? 0) - (c.longWinRate ?? 0);
	const eligible = $derived(
		data.cards.filter((c) => c.shortGames >= MIN_GAMES && c.longGames >= MIN_GAMES)
	);
	const lists = $derived(
		data.median == null
			? []
			: [
					{
						id: 'short',
						title: m.pace_short(),
						split: m.pace_short_split({ count: data.median }),
						cards: eligible
							.filter((c) => diff(c) > 0)
							.sort((a, b) => diff(b) - diff(a))
							.slice(0, 5)
					},
					{
						id: 'long',
						title: m.pace_long(),
						split: m.pace_long_split({ count: data.median }),
						cards: eligible
							.filter((c) => diff(c) < 0)
							.sort((a, b) => diff(a) - diff(b))
							.slice(0, 5)
					}
				]
	);
</script>

<section aria-labelledby="pace-title">
	<div class="grid gap-1 section-head">
		<h2 id="pace-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.pace_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.pace_text()}</p>
	</div>
	{#if data.median == null}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.no_stats()}</p>
	{:else}
		<figure class="grid gap-3 px-(--gutter) pt-2 pb-4" bind:clientWidth={width}>
			<Chart.Container {config} class="aspect-auto h-52 w-full">
				<BarChart data={rows} x="turns" {series} bandPadding={0.2} {padding} props={chartProps}>
					{#snippet tooltip()}
						<Chart.Tooltip labelFormatter={(v: number) => m.pace_turns_n({ count: v })} />
					{/snippet}
				</BarChart>
			</Chart.Container>
			<figcaption class="text-sm tabular-nums">{m.pace_median({ count: data.median })}</figcaption>
		</figure>
		<div class="border-t">
			<div class="cells grid sm:grid-cols-2">
				{#each lists as list (list.id)}
					<div class="grid content-start gap-3 px-(--gutter) py-4">
						<div class="grid gap-1">
							<h3 class="font-medium">{list.title}</h3>
							<p class="eyebrow text-muted-foreground">{list.split}</p>
						</div>
						{#if list.cards.length}
							<div
								class="grid grid-cols-[auto_minmax(0,1fr)_auto_auto] items-center gap-x-2 gap-y-2 text-sm tabular-nums"
							>
								<div
									class="col-span-4 grid grid-cols-subgrid eyebrow text-[10px] text-muted-foreground"
									aria-hidden="true"
								>
									<span class="col-start-3 text-right">{m.pace_short_rate()}</span>
									<span class="text-right">{m.pace_long_rate()}</span>
								</div>
								<ul class="col-span-4 grid grid-cols-subgrid gap-y-2">
									{#each list.cards as row (row.card)}
										{@const card = catalog.get(row.card)}
										<li class="col-span-4 grid grid-cols-subgrid items-center">
											{#if card}
												<img
													src="/art/avatar/{row.card}.webp"
													alt=""
													loading="lazy"
													decoding="async"
													class="el-{card.element} mr-1 size-8 rounded-xs border-2 border-(--el) bg-(--el)"
												/>
												<a href={href(`/cards/${row.card}`)} class="truncate hover:underline"
													>{card.name ?? row.card}</a
												>
											{:else}
												<span class="col-span-2 font-mono text-xs">{row.card.slice(0, 8)}</span>
											{/if}
											<span class="text-right"
												><span class="sr-only">{m.pace_short_rate()}</span>
												{percent(row.shortWinRate)}</span
											>
											<span class="text-right"
												><span class="sr-only">{m.pace_long_rate()}</span>
												{percent(row.longWinRate)}</span
											>
										</li>
									{/each}
								</ul>
							</div>
						{:else}
							<p class="text-sm text-muted-foreground">{m.pace_few({ min: MIN_GAMES })}</p>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	{/if}
</section>
