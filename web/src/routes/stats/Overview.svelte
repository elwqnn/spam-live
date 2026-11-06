<script lang="ts">
	import { BarChart } from 'layerchart';
	import type { Schemas } from '#lib';
	import * as Chart from '#lib/components/ui/chart/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';

	let { data }: { data: Schemas['Overview'] } = $props();

	const number = new Intl.NumberFormat(getLocale());
	const day = new Intl.DateTimeFormat(getLocale(), {
		day: 'numeric',
		month: 'short',
		timeZone: 'UTC'
	});
	const full = new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium', timeZone: 'UTC' });

	const cells = $derived([
		[m.overview_matches(), data.matches],
		[m.overview_players(), data.players],
		[m.overview_decks(), data.decks]
	] as const);
	const rows = $derived(data.days.map((d) => ({ ...d, date: new Date(d.day) })));
	let width = $state(0);
	const step = $derived(Math.ceil(rows.length / Math.max(2, Math.floor((width - 120) / 64))));
	const series = $derived([{ key: 'matches', label: m.overview_matches(), color: 'var(--moss)' }]);
	const config = $derived({ matches: { label: m.overview_matches(), color: 'var(--moss)' } });
	const yTicks = (scale: unknown) =>
		((scale as { ticks?: (n: number) => number[] }).ticks?.(4) ?? []).filter(Number.isInteger);
	const chartProps = $derived({
		xAxis: {
			ticks: () => rows.filter((_, i) => (rows.length - 1 - i) % step === 0).map((r) => r.date),
			format: (v: Date) => day.format(v)
		},
		yAxis: { ticks: yTicks, format: (v: number) => number.format(v) },
		bars: { radius: 0, strokeWidth: 0 }
	});
	const padding = { top: 8, right: 8, bottom: 28, left: 40 };
</script>

{#snippet tip({ value }: { value: unknown })}
	<span class="tabular-nums">{m.overview_day({ count: Number(value) })}</span>
{/snippet}

<section aria-labelledby="overview-title">
	<div class="grid gap-1 section-head">
		<h2 id="overview-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.overview_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.overview_text()}</p>
	</div>
	{#if data.matches}
		<div>
			<dl class="cells grid grid-cols-3">
				{#each cells as [name, value] (name)}
					<div class="grid content-start gap-1 px-(--gutter) py-4">
						<dt class="eyebrow text-[10px] text-muted-foreground">{name}</dt>
						<dd class="text-xl tabular-nums sm:text-3xl">{number.format(value)}</dd>
					</div>
				{/each}
			</dl>
		</div>
		<figure class="grid gap-3 border-t px-(--gutter) py-4" bind:clientWidth={width}>
			<figcaption class="eyebrow text-muted-foreground">{m.overview_chart()}</figcaption>
			<Chart.Container
				{config}
				role="img"
				aria-label={m.overview_chart_label()}
				class="aspect-auto h-44 w-full"
			>
				<BarChart data={rows} x="date" {series} bandPadding={0.2} {padding} props={chartProps}>
					{#snippet tooltip()}
						<Chart.Tooltip labelFormatter={(v: Date) => full.format(v)} formatter={tip} />
					{/snippet}
				</BarChart>
			</Chart.Container>
		</figure>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.no_stats()}</p>
	{/if}
</section>
