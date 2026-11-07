<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { InformationCircleIcon } from '@hugeicons/core-free-icons';
	import { BarChart } from 'layerchart';
	import { href, percent, winRateColor, type CardMap, type Schemas } from '#lib';
	import Tooltip from '#lib/components/Tooltip.svelte';
	import * as Chart from '#lib/components/ui/chart/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import Sparkline from './Sparkline.svelte';
	import { curveRates } from './spark.js';

	let { data, catalog }: { data: Schemas['PaceStats']; catalog: CardMap } = $props();

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

	const lists = $derived([
		{
			id: 'short',
			title: m.pace_short(),
			split:
				data.median == null ? m.pace_short_split_all() : m.pace_short_split({ count: data.median }),
			cards: data.short
		},
		{
			id: 'long',
			title: m.pace_long(),
			split:
				data.median == null ? m.pace_long_split_all() : m.pace_long_split({ count: data.median }),
			cards: data.long
		}
	]);
	let open = $state<Record<string, boolean>>({});
	const at = (rate: number) => `clamp(5px, ${rate * 100}%, calc(100% - 5px))`;
</script>

<section aria-labelledby="pace-title">
	<div class="grid gap-1 section-head">
		<h2 id="pace-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.pace_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.pace_text()}</p>
	</div>
	{#if !data.turns.length}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.no_stats()}</p>
	{:else}
		<figure class="grid gap-3 px-(--gutter) pt-2 pb-4" bind:clientWidth={width}>
			<Chart.Container {config} class="aspect-auto h-52 w-full sm:h-64">
				<BarChart data={rows} x="turns" {series} bandPadding={0.2} {padding} props={chartProps}>
					{#snippet tooltip()}
						<Chart.Tooltip labelFormatter={(v: number) => m.pace_turns_n({ count: v })} />
					{/snippet}
				</BarChart>
			</Chart.Container>
			<figcaption class="text-sm tabular-nums">
				{data.median == null ? m.pace_median_all() : m.pace_median({ count: data.median })}
			</figcaption>
		</figure>
		<div class="border-t">
			<div class="cells grid sm:grid-cols-2">
				{#each lists as list (list.id)}
					<div class="grid content-start gap-4 px-(--gutter) py-4">
						<div class="grid gap-1">
							<div class="flex items-center gap-1">
								<h3 class="font-medium">{list.title}</h3>
								<Tooltip content={m.pace_info()} class="max-w-72 p-3 text-sm leading-6">
									{#snippet children(tip)}
										<button
											type="button"
											{...tip}
											aria-label={m.pace_info_label()}
											aria-expanded={open[list.id] ?? false}
											aria-controls="pace-info-{list.id}"
											onclick={() => (open[list.id] = !open[list.id])}
											class="grid size-7 place-items-center rounded-sm text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-2 focus-visible:outline-offset-2"
										>
											<HugeiconsIcon
												icon={InformationCircleIcon}
												aria-hidden="true"
												class="size-4"
											/>
										</button>
									{/snippet}
								</Tooltip>
							</div>
							<p class="eyebrow text-muted-foreground">{list.split}</p>
							<p
								id="pace-info-{list.id}"
								class="mt-1 text-sm text-muted-foreground"
								hidden={!open[list.id]}
							>
								{m.pace_info()}
							</p>
						</div>
						{#if list.cards.length}
							<div class="grid gap-3">
								<div
									class="flex flex-wrap items-center justify-between gap-x-4 gap-y-1 text-xs text-muted-foreground"
									aria-hidden="true"
								>
									<span class="flex items-center gap-3">
										<span class="flex items-center gap-1.5"
											><span class="size-2.5 rounded-xs bg-terra"></span>{m.pace_short_rate()}</span
										>
										<span class="flex items-center gap-1.5"
											><span class="size-2.5 rounded-xs bg-slate"></span>{m.pace_long_rate()}</span
										>
									</span>
									<span>{m.pace_curve()}</span>
								</div>
								<ul class="grid gap-4">
									{#each list.cards as row (row.card)}
										{@const card = catalog.get(row.card)}
										{@const lo = Math.min(row.shortWinRate, row.longWinRate)}
										{@const hi = Math.max(row.shortWinRate, row.longWinRate)}
										<li class="grid gap-2">
											<div class="flex items-center gap-3 text-sm">
												{#if card}
													<img
														src="/art/avatar/{row.card}.webp"
														alt=""
														loading="lazy"
														decoding="async"
														class="el-{card.element} size-8 shrink-0 rounded-xs border-2 border-(--el) bg-(--el)"
													/>
													<a
														href={href(`/cards/${row.card}`)}
														class="line-clamp-2 min-w-[8ch] flex-1 leading-5 wrap-anywhere hover:underline"
														>{card.name ?? row.card}</a
													>
												{:else}
													<span class="min-w-[8ch] flex-1 font-mono text-xs"
														>{row.card.slice(0, 8)}</span
													>
												{/if}
												<span class="flex shrink-0 gap-3 tabular-nums">
													<span class="flex items-center gap-1.5">
														<span class="size-1.5 rounded-xs bg-terra" aria-hidden="true"></span>
														<span style:color={winRateColor(row.shortWinRate)}
															><span class="sr-only">{m.pace_short_rate()}</span>
															{percent(row.shortWinRate)}</span
														>
													</span>
													<span class="flex items-center gap-1.5">
														<span class="size-1.5 rounded-xs bg-slate" aria-hidden="true"></span>
														<span style:color={winRateColor(row.longWinRate)}
															><span class="sr-only">{m.pace_long_rate()}</span>
															{percent(row.longWinRate)}</span
														>
													</span>
												</span>
											</div>
											<div class="flex items-center gap-4 pl-11">
												<span
													class="relative block h-1.5 min-w-0 flex-1 rounded-[2px] bg-foreground/6 sm:max-w-40"
													aria-hidden="true"
												>
													<span class="absolute -inset-y-1 left-1/2 w-px bg-foreground/40"></span>
													<span
														class="absolute top-1/2 h-0.5 -translate-y-1/2 bg-muted-foreground"
														style:left={at(lo)}
														style:right="calc(100% - {at(hi)})"
													></span>
													<span
														class="absolute top-1/2 size-2.5 -translate-1/2 rounded-xs bg-terra"
														style:left={at(row.shortWinRate)}
													></span>
													<span
														class="absolute top-1/2 size-2.5 -translate-1/2 rounded-xs bg-slate"
														style:left={at(row.longWinRate)}
													></span>
												</span>
												<Sparkline
													values={curveRates(row.curve)}
													range={[0.2, 0.8]}
													mid={0.5}
													class="text-muted-foreground"
												/>
											</div>
										</li>
									{/each}
								</ul>
							</div>
						{:else}
							<p class="text-sm text-muted-foreground">{m.pace_few()}</p>
						{/if}
					</div>
				{/each}
			</div>
		</div>
	{/if}
</section>
