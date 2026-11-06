<script lang="ts">
	import { ScatterChart, Tooltip } from 'layerchart';
	import { href, percent, type CardMap, type Schemas } from '#lib';
	import { ELEMENTS } from '#lib/components/CardBadge.svelte';
	import * as Chart from '#lib/components/ui/chart/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import ElementLegend from './ElementLegend.svelte';
	import { rateDomain } from './turns.js';

	let { data, catalog }: { data: Schemas['CardStats'][]; catalog: CardMap } = $props();

	type Dot = {
		card: string;
		name: string;
		element: string;
		playRate: number;
		winRate: number;
		games: number;
	};

	const MIN_GAMES = 20;
	const number = new Intl.NumberFormat(getLocale());

	const dots: Dot[] = $derived(
		data
			.filter((s) => s.games >= MIN_GAMES && s.winRate != null)
			.map((s) => {
				const card = catalog.get(s.card);
				const element = card && Object.hasOwn(ELEMENTS, card.element) ? card.element : '';
				return {
					card: s.card,
					name: card?.name ?? s.card.slice(0, 8),
					element,
					playRate: s.playRate,
					winRate: s.winRate ?? 0,
					games: s.games
				};
			})
	);
	const hidden = $derived(data.length - dots.length);
	const series = $derived(
		[...Object.keys(ELEMENTS), ''].map((key) => ({
			key: key || 'unknown',
			label: key ? ELEMENTS[key]() : m.unknown_card(),
			color: key ? `var(--el-${key})` : 'var(--foreground)',
			data: dots.filter((d) => d.element === key)
		}))
	);
	const config = $derived(
		Object.fromEntries(series.map((s) => [s.key, { label: s.label, color: s.color }]))
	);
	const median = $derived.by(() => {
		const rates = dots.map((d) => d.playRate).sort((a, b) => a - b);
		const mid = Math.floor(rates.length / 2);
		return rates.length % 2 ? rates[mid] : (rates[mid - 1] + rates[mid]) / 2;
	});
	const xDomain = $derived([0, Math.max(...dots.map((d) => d.playRate)) * 1.1]);
	const yDomain = $derived(rateDomain([...dots.map((d) => d.winRate), 0.5]));
	const rule = { class: 'stroke-muted-foreground/50! [stroke-dasharray:4_4]' };
	const annotations = $derived([
		{ type: 'line' as const, y: 0.5, props: { line: rule } },
		{ type: 'line' as const, x: median, props: { line: rule } }
	]);
	const xTicks = (scale: unknown) =>
		((scale as { ticks?: (n: number) => number[] }).ticks?.(5) ?? []).filter((v) => v > 0);
	const chartProps = $derived({
		xAxis: {
			ticks: xTicks,
			format: (v: number) => percent(v),
			label: m.play_rate(),
			labelPlacement: 'end' as const
		},
		yAxis: {
			ticks: 5,
			format: (v: number) => percent(v),
			label: m.win_rate(),
			labelPlacement: 'start' as const
		},
		points: { r: 5, stroke: 'var(--background)', strokeWidth: 1 }
	});
	const corner = 'pointer-events-none absolute eyebrow text-[10px] text-muted-foreground';
</script>

<section aria-labelledby="metamap-title">
	<div class="grid gap-1 section-head">
		<h2 id="metamap-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.metamap_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.metamap_text({ min: MIN_GAMES })}</p>
	</div>
	{#if dots.length}
		<ElementLegend />
		<figure class="grid gap-3 px-(--gutter) py-4">
			<div role="img" aria-label={m.metamap_label()} class="relative h-80 sm:h-96">
				<Chart.Container {config} class="aspect-auto h-full w-full">
					<ScatterChart
						x="playRate"
						y="winRate"
						{series}
						{xDomain}
						{yDomain}
						{annotations}
						padding={{ top: 24, right: 8, bottom: 40, left: 44 }}
						props={chartProps}
					>
						{#snippet tooltip()}
							<Tooltip.Root variant="none">
								{#snippet children({ data: dot }: { data: Dot })}
									<div
										class="grid min-w-36 gap-1 rounded-sm border bg-popover px-2.5 py-1.5 text-xs text-popover-foreground"
									>
										<div class="font-medium">{dot.name}</div>
										{#each [[m.win_rate(), percent(dot.winRate)], [m.play_rate(), percent(dot.playRate)], [m.games(), number.format(dot.games)]] as [label, value] (label)}
											<div class="flex justify-between gap-4">
												<span class="text-muted-foreground">{label}</span>
												<span class="font-medium tabular-nums">{value}</span>
											</div>
										{/each}
									</div>
								{/snippet}
							</Tooltip.Root>
						{/snippet}
					</ScatterChart>
				</Chart.Container>
				<span class="{corner} top-8 right-3">{m.metamap_picks()}</span>
				<span class="{corner} top-8 left-14">{m.metamap_underplayed()}</span>
				<span class="{corner} right-3 bottom-13">{m.metamap_overplayed()}</span>
				<span class="{corner} bottom-13 left-14">{m.metamap_weak()}</span>
			</div>
			<figcaption class="text-sm text-muted-foreground">
				{#if hidden}{m.metamap_hidden({ count: hidden, min: MIN_GAMES })}{/if}
				<a href={href('/cards')} class="sr-only focus-visible:not-sr-only focus-visible:underline"
					>{m.metamap_list()}</a
				>
			</figcaption>
		</figure>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.no_stats()}</p>
	{/if}
</section>
