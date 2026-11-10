<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowRight01Icon } from '@hugeicons/core-free-icons';
	import { BarChart, LineChart } from 'layerchart';
	import type { Schemas } from '#lib';
	import { hourLabel, hourTicks, type Range } from '#lib/analytics.js';
	import FilterMenu from '@spam-live/ui/components/FilterMenu.svelte';
	import * as Chart from '@spam-live/ui/components/ui/chart/index.js';
	import * as Table from '@spam-live/ui/components/ui/table/index.js';
	import type { Filter } from '@spam-live/ui/filter.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';

	let { data, range }: { data: Schemas['Analytics']; range: Range } = $props();

	type Row = Schemas['HourStats'] & { date: Date };
	type Key = 'webPeak' | 'appPeak' | 'matchesAdded' | 'matches';
	type Series = { key: Key; label: string; color: string; swatch: string };

	const number = new Intl.NumberFormat(getLocale());
	const num = (n: number | null | undefined) => (n == null ? '—' : number.format(n));
	const fullHour = new Intl.DateTimeFormat(getLocale(), {
		dateStyle: 'medium',
		timeStyle: 'short',
		timeZone: 'UTC'
	});

	const ranges: Filter = $derived({
		id: 'range',
		label: m.analytics_range(),
		default: '7d',
		options: [
			{ value: '24h', label: m.range_24h() },
			{ value: '7d', label: m.trend_week() },
			{ value: '30d', label: m.range_30d() }
		]
	});

	const sections = $derived([
		{
			id: 'now',
			title: m.analytics_now(),
			cells: [
				[m.analytics_web_online(), data.now.webOnline],
				[m.analytics_app_online(), data.now.appOnline]
			]
		},
		{
			id: 'matches',
			title: m.analytics_matches(),
			cells: [
				[m.analytics_total(), data.matches.total],
				[m.analytics_ranked(), data.matches.ranked],
				[m.analytics_ai(), data.matches.ai],
				[m.analytics_disputed(), data.matches.disputed],
				[m.analytics_last24h(), data.matches.last24h],
				[m.analytics_last7d(), data.matches.last7d]
			]
		},
		{
			id: 'players',
			title: m.analytics_players(),
			cells: [
				[m.analytics_accounts(), data.players.accounts],
				[m.analytics_linked(), data.players.linkedAccounts],
				[m.analytics_banned(), data.players.banned],
				[m.analytics_signups(), data.players.signups7d]
			]
		},
		{
			id: 'community',
			title: m.analytics_community(),
			cells: [
				[m.analytics_public_decks(), data.community.publicDecks],
				[m.analytics_drafts(), data.community.draftDecks],
				[m.comments(), data.community.comments],
				[m.votes(), data.community.votes]
			]
		},
		{
			id: 'desktop',
			title: m.devices(),
			cells: [
				[m.analytics_devices(), data.desktop.devices],
				[m.analytics_seen(), data.desktop.seen24h]
			]
		},
		{
			id: 'uploads',
			title: m.analytics_uploads(),
			cells: [
				[m.website(), data.uploads.web],
				[m.download_title(), data.uploads.desktop],
				[m.analytics_unknown(), data.uploads.unknown]
			]
		}
	] as const);

	const rows: Row[] = $derived(data.hours.map((h) => ({ ...h, date: new Date(h.hour) })));
	let width = $state(0);
	const ticks = $derived(
		hourTicks(
			rows.map((r) => r.date),
			range,
			Math.max(2, Math.floor((width - 120) / 64))
		)
	);
	const tickLabel = $derived(hourLabel(range, getLocale()));

	const charts: { id: string; title: string; bar: boolean; series: Series[] }[] = $derived([
		{
			id: 'peaks',
			title: m.analytics_peaks(),
			bar: false,
			series: [
				{ key: 'webPeak', label: m.website(), color: 'var(--terra)', swatch: 'bg-terra' },
				{ key: 'appPeak', label: m.download_title(), color: 'var(--slate)', swatch: 'bg-slate' }
			]
		},
		{
			id: 'added',
			title: m.analytics_added(),
			bar: true,
			series: [
				{
					key: 'matchesAdded',
					label: m.analytics_added_short(),
					color: 'var(--straw)',
					swatch: 'bg-straw'
				}
			]
		},
		{
			id: 'total',
			title: m.analytics_total_matches(),
			bar: false,
			series: [
				{ key: 'matches', label: m.analytics_total(), color: 'var(--moss)', swatch: 'bg-moss' }
			]
		}
	]);

	const yTicks = (scale: unknown) =>
		((scale as { ticks?: (n: number) => number[] }).ticks?.(4) ?? []).filter(Number.isInteger);
	const chartProps = $derived({
		xAxis: {
			ticks: () => ticks,
			format: tickLabel,
			label: 'UTC',
			labelPlacement: 'end' as const
		},
		yAxis: { ticks: yTicks, format: (v: number) => number.format(v) },
		bars: { radius: 0, strokeWidth: 0 },
		spline: { strokeWidth: 2 }
	});
	const padding = { top: 8, right: 8, bottom: 36, left: 48 };
	const config = (series: Series[]) =>
		Object.fromEntries(series.map((s) => [s.key, { label: s.label, color: s.color }]));
	const empty = (series: Series[]) => !rows.some((r) => series.some((s) => r[s.key] != null));
	const tooltipLabel = (value: Date) => `${fullHour.format(value)} UTC`;
	const table = $derived([...rows].reverse());
</script>

<div class="px-(--gutter) py-3">
	<FilterMenu filter={ranges} />
</div>

{#each sections as section (section.id)}
	<section aria-labelledby="analytics-{section.id}">
		<h2
			id="analytics-{section.id}"
			class="section-head font-heading text-2xl font-normal tracking-tight"
		>
			{section.title}
		</h2>
		<div>
			<dl class="cells grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6">
				{#each section.cells as [name, value] (name)}
					<div class="grid content-start gap-1 px-(--gutter) py-4">
						<dt class="eyebrow text-[10px] text-muted-foreground">{name}</dt>
						<dd class="text-xl tabular-nums">{num(value)}</dd>
					</div>
				{/each}
			</dl>
		</div>
		{#if section.id === 'desktop'}
			<div class="border-t">
				<h3 class="px-(--gutter) pt-4 pb-2 eyebrow text-[10px] text-muted-foreground">
					{m.analytics_versions()}
				</h3>
				{#if data.desktop.versions.length}
					<Table.Root>
						<Table.Header>
							<Table.Row>
								<Table.Head class="pl-(--gutter)">{m.version()}</Table.Head>
								<Table.Head class="pr-(--gutter) text-right">{m.analytics_apps()}</Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each data.desktop.versions as v (v.version)}
								<Table.Row>
									<Table.Cell class="pl-(--gutter)">{v.version ?? m.analytics_unknown()}</Table.Cell
									>
									<Table.Cell class="pr-(--gutter) text-right tabular-nums"
										>{num(v.devices)}</Table.Cell
									>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				{:else}
					<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">
						{m.analytics_no_versions()}
					</p>
				{/if}
			</div>
		{/if}
	</section>
{/each}

<section aria-labelledby="analytics-history">
	<h2 id="analytics-history" class="section-head font-heading text-2xl font-normal tracking-tight">
		{m.analytics_history()}
	</h2>
	<div class="divide-y" bind:clientWidth={width}>
		{#each charts as chart (chart.id)}
			<figure class="grid gap-3 px-(--gutter) py-4">
				<figcaption class="flex flex-wrap items-center gap-x-4 gap-y-1">
					<span class="eyebrow text-muted-foreground">{chart.title}</span>
					{#if chart.series.length > 1}
						{#each chart.series as s (s.key)}
							<span class="flex items-center gap-1.5 text-xs text-muted-foreground">
								<span class={['size-2.5 rounded-xs', s.swatch]} aria-hidden="true"></span>
								{s.label}
							</span>
						{/each}
					{/if}
				</figcaption>
				{#if empty(chart.series)}
					<p class="text-sm text-muted-foreground">{m.analytics_no_data()}</p>
				{:else}
					<Chart.Container config={config(chart.series)} class="aspect-auto h-52 w-full">
						{#if chart.bar}
							<BarChart
								data={rows}
								x="date"
								series={chart.series}
								bandPadding={0.2}
								{padding}
								props={chartProps}
							>
								{#snippet tooltip()}
									<Chart.Tooltip labelFormatter={tooltipLabel} />
								{/snippet}
							</BarChart>
						{:else}
							<LineChart data={rows} x="date" series={chart.series} {padding} props={chartProps}>
								{#snippet tooltip()}
									<Chart.Tooltip labelFormatter={tooltipLabel} />
								{/snippet}
							</LineChart>
						{/if}
					</Chart.Container>
				{/if}
			</figure>
		{/each}
	</div>
	<details class="group border-t">
		<summary
			class="flex cursor-pointer list-none items-center gap-2 px-(--gutter) py-3 text-sm [&::-webkit-details-marker]:hidden"
		>
			<HugeiconsIcon
				icon={ArrowRight01Icon}
				aria-hidden="true"
				class="size-4 text-muted-foreground transition-transform group-open:rotate-90 motion-reduce:transition-none"
			/>
			{m.analytics_hourly()}
		</summary>
		<Table.Root>
			<Table.Header>
				<Table.Row>
					<Table.Head class="pl-(--gutter)">{m.analytics_hour()}</Table.Head>
					<Table.Head class="text-right">{m.analytics_web_peak()}</Table.Head>
					<Table.Head class="text-right">{m.analytics_app_peak()}</Table.Head>
					<Table.Head class="text-right">{m.analytics_web_distinct()}</Table.Head>
					<Table.Head class="text-right">{m.analytics_app_distinct()}</Table.Head>
					<Table.Head class="text-right">{m.analytics_added_short()}</Table.Head>
					<Table.Head class="pr-(--gutter) text-right">{m.analytics_total_matches()}</Table.Head>
				</Table.Row>
			</Table.Header>
			<Table.Body>
				{#each table as h (h.hour)}
					<Table.Row>
						<Table.Cell class="pl-(--gutter)"
							><time datetime={h.hour}>{fullHour.format(h.date)}</time></Table.Cell
						>
						<Table.Cell class="text-right tabular-nums">{num(h.webPeak)}</Table.Cell>
						<Table.Cell class="text-right tabular-nums">{num(h.appPeak)}</Table.Cell>
						<Table.Cell class="text-right tabular-nums">{num(h.webDistinct)}</Table.Cell>
						<Table.Cell class="text-right tabular-nums">{num(h.appDistinct)}</Table.Cell>
						<Table.Cell class="text-right tabular-nums">{num(h.matchesAdded)}</Table.Cell>
						<Table.Cell class="pr-(--gutter) text-right tabular-nums">{num(h.matches)}</Table.Cell>
					</Table.Row>
				{/each}
			</Table.Body>
		</Table.Root>
	</details>
</section>
