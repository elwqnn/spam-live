<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowExpand01Icon, ZoomInAreaIcon, ZoomOutAreaIcon } from '@hugeicons/core-free-icons';
	import { Html, ScatterChart, type ChartState } from 'layerchart';
	import { percent, winRateColor, type CardMap, type Schemas } from '#ui';
	import { art, href } from '$host';
	import { ELEMENTS } from '#ui/components/CardBadge.svelte';
	import { Button } from '#ui/components/ui/button/index.js';
	import * as Chart from '#ui/components/ui/chart/index.js';
	import { m } from '$paraglide/messages.js';
	import { getLocale } from '$paraglide/runtime.js';
	import ElementLegend from './ElementLegend.svelte';
	import { rateDomain } from './domain.js';

	let { data, catalog }: { data: Schemas['CardStats'][]; catalog: CardMap } = $props();

	type Dot = {
		card: string;
		name: string;
		element: string | null;
		playRate: number;
		winRate: number;
		games: number;
	};

	const MAX_ZOOM = 8;
	const TIP = 208;
	const number = new Intl.NumberFormat(getLocale());

	const dots: Dot[] = $derived(
		data
			.filter((s) => s.winRate != null)
			.map((s) => {
				const card = catalog.get(s.card);
				return {
					card: s.card,
					name: card?.name ?? s.card.slice(0, 8),
					element: card ? card.element : null,
					playRate: s.playRate,
					winRate: s.winRate ?? 0,
					games: s.games
				};
			})
	);
	const config = $derived(
		Object.fromEntries(
			Object.keys(ELEMENTS).map((key) => [
				key,
				{ label: ELEMENTS[key](), color: `var(--el-${key})` }
			])
		)
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
			format: (v: number) => percent(v)
		}
	});
	const padding = { top: 44, right: 8, bottom: 40, left: 44 };

	let context = $state<ChartState>();
	let scale = $state(1);
	let active = $state.raw<Dot | null>(null);
	let tipHeight = $state(0);
	const zoomed = $derived(scale > 1.001);
	const size = $derived(scale >= 4 ? 32 : 26);
	const transform = $derived({
		mode: 'domain' as const,
		scaleExtent: [1, MAX_ZOOM] as [number, number],
		domainExtent: { x: { min: 'data', max: 'data' }, y: { min: 'data', max: 'data' } } as const,
		scrollActivationKey: 'control' as const,
		disablePointer: !zoomed
	});
	const tools = $derived([
		{
			label: m.metamap_zoom_out(),
			icon: ZoomOutAreaIcon,
			off: !zoomed,
			run: () => context?.transformState?.zoomOut()
		},
		{
			label: m.metamap_zoom_in(),
			icon: ZoomInAreaIcon,
			off: scale >= MAX_ZOOM - 0.001,
			run: () => context?.transformState?.zoomIn()
		},
		{
			label: m.metamap_zoom_reset(),
			icon: ArrowExpand01Icon,
			off: !zoomed,
			run: () => context?.transformState?.reset()
		}
	]);
	const show = (dot: Dot) => (active = dot);
	const hide = (dot: Dot) => {
		if (active === dot) active = null;
	};
	const corner = 'pointer-events-none absolute z-1 eyebrow text-[10px] text-muted-foreground';
</script>

<section aria-labelledby="metamap-title">
	<div class="grid gap-1 section-head">
		<h2 id="metamap-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.metamap_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.metamap_text()}</p>
	</div>
	{#if dots.length}
		<ElementLegend />
		<figure class="grid gap-3 px-(--gutter) py-4">
			<div
				role="group"
				aria-label={m.metamap_label()}
				class="relative h-96 sm:h-[34rem] lg:h-[40rem]"
			>
				<span class="absolute top-0 left-0 flex h-8 items-center text-xs text-muted-foreground"
					>{m.win_rate()}</span
				>
				<div class="absolute top-0 right-0 z-1 flex items-center gap-1">
					<span class="mr-2 hidden text-xs text-muted-foreground sm:inline pointer-coarse:hidden"
						>{m.metamap_zoom_hint()}</span
					>
					{#each tools as tool (tool.label)}
						<Button
							variant="outline"
							size="icon-sm"
							aria-label={tool.label}
							aria-disabled={tool.off ? 'true' : undefined}
							class="aria-disabled:cursor-default aria-disabled:opacity-50"
							onclick={() => tool.off || tool.run()}
						>
							<HugeiconsIcon icon={tool.icon} aria-hidden="true" class="size-4" />
						</Button>
					{/each}
				</div>
				<Chart.Container {config} class="aspect-auto h-full w-full">
					<ScatterChart
						data={dots}
						x="playRate"
						y="winRate"
						{xDomain}
						{yDomain}
						{annotations}
						{padding}
						{transform}
						tooltipContext={false}
						props={chartProps}
						onTransform={(t) => (scale = t.scale)}
						bind:context
					>
						{#snippet marks()}{/snippet}
						{#snippet aboveContext({ context: c })}
							<Html clip={false} class="overflow-clip">
								{#each dots as dot (dot.card)}
									{@const x = c.xScale(dot.playRate)}
									{@const y = c.yScale(dot.winRate)}
									{#if x >= 0 && x <= c.width && y >= 0 && y <= c.height}
										<a
											href={href(`/cards/${dot.card}`)}
											draggable="false"
											class={[
												'group absolute -translate-1/2 rounded-xs focus-visible:z-10 focus-visible:outline-2 focus-visible:outline-offset-2',
												active === dot && 'z-10'
											]}
											style:left="{x}px"
											style:top="{y}px"
											onpointerenter={() => show(dot)}
											onpointerleave={() => hide(dot)}
											onfocus={(e) => e.currentTarget.matches(':focus-visible') && show(dot)}
											onblur={() => hide(dot)}
										>
											{#if dot.element != null}
												<img
													src={art(`avatar/${dot.card}.webp`)}
													alt=""
													loading="lazy"
													decoding="async"
													draggable="false"
													class="el-{dot.element} block rounded-xs border-2 border-(--el) bg-(--el) transition-colors group-hover:border-foreground"
													style:width="{size}px"
													style:height="{size}px"
												/>
											{:else}
												<span class="block size-2.5 rounded-xs bg-foreground"></span>
											{/if}
											<span class="sr-only">{dot.name}</span>
										</a>
									{/if}
								{/each}
							</Html>
							{#if active}
								{@const x = c.xScale(active.playRate)}
								{@const y = c.yScale(active.winRate)}
								<Html clip={false} pointerEvents={false}>
									<div
										class="absolute grid gap-2 rounded-sm border bg-popover p-2.5 text-xs text-popover-foreground"
										style:width="{TIP}px"
										style:left="{Math.max(
											-padding.left,
											padding.left + x + 16 + TIP > c.containerWidth ? x - 16 - TIP : x + 16
										)}px"
										style:top="{Math.min(
											Math.max(-padding.top, y - tipHeight / 2),
											c.height + padding.bottom - tipHeight
										)}px"
										bind:offsetHeight={tipHeight}
										aria-hidden="true"
									>
										<div class="flex items-center gap-2.5">
											{#if active.element != null}
												<img
													src={art(`avatar/${active.card}.webp`)}
													alt=""
													decoding="async"
													class="el-{active.element} size-12 shrink-0 rounded-xs border-2 border-(--el) bg-(--el)"
												/>
											{/if}
											<span class="text-sm font-medium">{active.name}</span>
										</div>
										<dl class="grid gap-1 tabular-nums">
											{#each [[m.win_rate(), percent(active.winRate), winRateColor(active.winRate)], [m.play_rate(), percent(active.playRate), undefined], [m.games(), number.format(active.games), undefined]] as [label, value, color] (label)}
												<div class="flex justify-between gap-4">
													<dt class="text-muted-foreground">{label}</dt>
													<dd class="font-medium" style:color>{value}</dd>
												</div>
											{/each}
										</dl>
									</div>
								</Html>
							{/if}
						{/snippet}
					</ScatterChart>
				</Chart.Container>
				<span class="{corner} top-13 right-3">{m.metamap_picks()}</span>
				<span class="{corner} top-13 left-14">{m.metamap_underplayed()}</span>
				<span class="{corner} right-3 bottom-13">{m.metamap_overplayed()}</span>
				<span class="{corner} bottom-13 left-14">{m.metamap_weak()}</span>
			</div>
			<figcaption class="text-sm text-muted-foreground">
				<a href={href('/cards')} class="sr-only focus-visible:not-sr-only focus-visible:underline"
					>{m.metamap_list()}</a
				>
			</figcaption>
		</figure>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.no_stats()}</p>
	{/if}
</section>
