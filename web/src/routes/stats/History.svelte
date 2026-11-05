<script lang="ts">
	import { LineChart } from 'layerchart';
	import { percent, type Schemas } from '#lib';
	import { ELEMENTS } from '#lib/components/CardBadge.svelte';
	import * as Chart from '#lib/components/ui/chart/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';

	let { data }: { data: Schemas['HistoryStats'] } = $props();

	type Row = { date: Date; [element: string]: Date | number | null };

	const keys = Object.keys(ELEMENTS);
	const day = new Intl.DateTimeFormat(getLocale(), {
		day: 'numeric',
		month: 'short',
		timeZone: 'UTC'
	});
	const full = new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium', timeZone: 'UTC' });

	const charts = $derived.by(() => {
		const share: Row[] = [];
		const rate: Row[] = [];
		for (const week of new Set(data.weeks.map((w) => w.week))) {
			const items = data.weeks.filter((w) => w.week === week);
			const date = new Date(week);
			const total = items.reduce((n, w) => n + w.games, 0);
			const find = (key: string) => items.find((w) => w.element === key);
			share.push({
				date,
				...Object.fromEntries(keys.map((k) => [k, (find(k)?.games ?? 0) / total]))
			});
			rate.push({ date, ...Object.fromEntries(keys.map((k) => [k, find(k)?.winRate ?? null])) });
		}
		const values = rate.flatMap((r) => keys.map((k) => r[k])).filter((v) => typeof v === 'number');
		const lo = Math.max(0, Math.floor(Math.min(...values) * 20 - 1)) / 20;
		const hi = Math.min(20, Math.ceil(Math.max(...values) * 20 + 1)) / 20;
		return [
			{ id: 'share', title: m.history_share(), rows: share, domain: undefined },
			{ id: 'rate', title: m.win_rate(), rows: rate, domain: values.length ? [lo, hi] : undefined }
		];
	});
	const dates = $derived(charts[0].rows.map((r) => r.date));
	const end = $derived(dates.length ? dates[dates.length - 1].getTime() + 7 * 864e5 : 0);

	let width = $state(0);
	const step = $derived(Math.ceil(dates.length / Math.max(2, Math.floor((width - 120) / 64))));
	const series = keys.map((k) => ({ key: k, label: ELEMENTS[k](), color: `var(--el-${k})` }));
	const config = Object.fromEntries(series.map((s) => [s.key, { label: s.label, color: s.color }]));
	const chartProps = $derived({
		xAxis: {
			ticks: () => dates.filter((_, i) => i % step === 0),
			format: (v: Date) => day.format(v)
		},
		yAxis: { ticks: 4, format: (v: number) => percent(v) },
		spline: { strokeWidth: 2 }
	});
	const starts = $derived(
		data.versions
			.map((v) => ({ version: v.version, date: new Date(v.start) }))
			.filter((v) => dates.length && v.date >= dates[0] && v.date.getTime() < end)
	);
	const xDomain = $derived([
		dates[0],
		new Date(Math.max(+dates[dates.length - 1], ...starts.map((v) => +v.date)))
	]);
	const annotations = $derived(
		starts.map((v) => ({
			type: 'line' as const,
			x: v.date,
			label: v.version,
			labelPlacement:
				+v.date - +xDomain[0] > 0.85 * (+xDomain[1] - +xDomain[0])
					? ('top-left' as const)
					: ('top-right' as const),
			labelXOffset: 4,
			props: {
				line: { class: 'stroke-muted-foreground/50! [stroke-dasharray:4_4]' },
				label: { class: 'fill-muted-foreground' }
			}
		}))
	);
	const padding = { top: 8, right: 8, bottom: 36, left: 48 };
</script>

<section aria-labelledby="history-title">
	<div class="grid gap-1 section-head">
		<h2 id="history-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.history_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.history_text()}</p>
		<p class="mt-1 eyebrow text-muted-foreground">{m.history_note()}</p>
	</div>
	{#if dates.length < 2}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.history_few()}</p>
	{:else}
		<ul class="flex flex-wrap gap-x-4 gap-y-1 px-(--gutter) pt-2">
			{#each series as s (s.key)}
				<li class="flex items-center gap-1.5 text-xs text-muted-foreground">
					<img
						src="/art/ui/element_{s.key}.webp"
						alt=""
						aria-hidden="true"
						decoding="async"
						class="size-4"
					/>
					{s.label}
				</li>
			{/each}
		</ul>
		<div class="divide-y" bind:clientWidth={width}>
			{#each charts as chart (chart.id)}
				<figure class="grid gap-3 px-(--gutter) py-4">
					<figcaption class="eyebrow text-muted-foreground">{chart.title}</figcaption>
					<Chart.Container {config} class="aspect-auto h-60 w-full">
						<LineChart
							data={chart.rows}
							x="date"
							{series}
							{padding}
							{annotations}
							{xDomain}
							yDomain={chart.domain}
							yBaseline={chart.domain ? null : 0}
							props={chartProps}
						>
							{#snippet tooltip()}
								<Chart.Tooltip
									labelFormatter={(v: Date) => m.history_week({ date: full.format(v) })}
									valueFormatter={percent}
								/>
							{/snippet}
						</LineChart>
					</Chart.Container>
				</figure>
			{/each}
		</div>
	{/if}
</section>
