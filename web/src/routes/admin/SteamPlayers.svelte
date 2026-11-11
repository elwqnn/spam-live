<script lang="ts">
	import { LineChart } from 'layerchart';
	import { STEAM_STORE, type Schemas } from '#lib';
	import { hourLabel, hourTicks } from '#lib/analytics.js';
	import * as Chart from '@spam-live/ui/components/ui/chart/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';

	let { hourly }: { hourly: Schemas['PlayerHour'][] } = $props();

	const number = new Intl.NumberFormat(getLocale());
	const fullHour = new Intl.DateTimeFormat(getLocale(), {
		dateStyle: 'medium',
		timeStyle: 'short',
		timeZone: 'UTC'
	});

	const rows = $derived(hourly.map((h) => ({ ...h, date: new Date(h.hour) })));
	let width = $state(0);
	const ticks = $derived(
		hourTicks(
			rows.map((r) => r.date),
			'7d',
			Math.max(2, Math.floor((width - 120) / 64))
		)
	);
	const line = $derived({ label: m.analytics_steam_players(), color: 'var(--terra)' });
	const series = $derived([{ key: 'max', ...line }]);
	const config = $derived({ max: line });
	const yTicks = (scale: unknown) =>
		((scale as { ticks?: (n: number) => number[] }).ticks?.(4) ?? []).filter(Number.isInteger);
	const chartProps = $derived({
		xAxis: {
			ticks: () => ticks,
			format: hourLabel('7d', getLocale()),
			label: 'UTC',
			labelPlacement: 'end' as const
		},
		yAxis: { ticks: yTicks, format: (v: number) => number.format(v) },
		spline: { strokeWidth: 2 }
	});
	const padding = { top: 8, right: 8, bottom: 36, left: 48 };
	const tooltipLabel = (value: Date) => `${fullHour.format(value)} UTC`;
</script>

<section aria-labelledby="analytics-steam">
	<h2 id="analytics-steam" class="section-head font-heading text-2xl font-normal tracking-tight">
		{m.analytics_steam_players()}
	</h2>
	<figure class="grid gap-3 px-(--gutter) py-4" bind:clientWidth={width}>
		<figcaption class="text-xs text-muted-foreground">
			<a
				class="underline underline-offset-4 transition-colors hover:text-foreground"
				href={STEAM_STORE}
				target="_blank"
				rel="external noopener">{m.steam_data()}</a
			>
		</figcaption>
		{#if rows.some((r) => r.max != null)}
			<Chart.Container {config} class="aspect-auto h-52 w-full">
				<LineChart data={rows} x="date" {series} {padding} props={chartProps}>
					{#snippet tooltip()}
						<Chart.Tooltip labelFormatter={tooltipLabel} />
					{/snippet}
				</LineChart>
			</Chart.Container>
		{:else}
			<p class="text-sm text-muted-foreground">{m.analytics_no_data()}</p>
		{/if}
	</figure>
</section>
