<script lang="ts">
	import { LineChart } from 'layerchart';
	import { href, percent, type CardMap, type Schemas } from '#lib';
	import { ELEMENTS } from '#lib/components/CardBadge.svelte';
	import * as Chart from '#lib/components/ui/chart/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { buckets, MIN_GAMES, rateDomain } from './turns.js';

	let { data, catalog }: { data: Schemas['TurnStats']; catalog: CardMap } = $props();

	const cards = $derived(buckets(data.cards));
	const labels = $derived(cards[0]?.points.map((p) => p.label) ?? []);
	const yDomain = $derived(
		rateDomain([
			...cards.flatMap((c) => c.points.map((p) => p.winRate).filter((v) => v != null)),
			0.5
		])
	);

	let width = $state(0);
	const step = $derived(Math.ceil(labels.length / Math.max(2, Math.floor((width - 112) / 44))));
	const chartProps = $derived({
		xAxis: { ticks: () => labels.filter((_, i) => i % step === 0) },
		yAxis: { ticks: 3, format: (v: number) => percent(v) },
		spline: { strokeWidth: 2 },
		points: { r: 3 }
	});
	const annotations = [
		{
			type: 'line' as const,
			y: 0.5,
			props: { line: { class: 'stroke-muted-foreground/50! [stroke-dasharray:4_4]' } }
		}
	];
	const padding = { top: 8, right: 8, bottom: 24, left: 40 };
	const color = (element?: string) =>
		element && Object.hasOwn(ELEMENTS, element) ? `var(--el-${element})` : 'var(--foreground)';
</script>

<section aria-labelledby="turns-title">
	<div class="grid gap-1 section-head">
		<h2 id="turns-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.turns_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.turns_text({ min: MIN_GAMES })}</p>
	</div>
	{#if cards.length}
		<div>
			<ul class="cells grid sm:grid-cols-2 lg:grid-cols-3">
				{#each cards as row (row.card)}
					{@const card = catalog.get(row.card)}
					{@const series = [{ key: 'winRate', label: m.win_rate(), color: color(card?.element) }]}
					<li class="grid content-start gap-3 px-(--gutter) py-4" bind:clientWidth={width}>
						<div class="flex items-center gap-3 text-sm">
							{#if card}
								<img
									src="/art/avatar/{row.card}.webp"
									alt=""
									loading="lazy"
									decoding="async"
									class="el-{card.element} size-8 shrink-0 rounded-xs border-2 border-(--el) bg-(--el)"
								/>
								<a href={href(`/cards/${row.card}`)} class="min-w-0 truncate hover:underline"
									>{card.name ?? row.card}</a
								>
							{:else}
								<span class="font-mono text-xs">{row.card.slice(0, 8)}</span>
							{/if}
						</div>
						{#if row.points.some((p) => p.winRate != null)}
							<Chart.Container
								config={{ winRate: { label: m.win_rate(), color: series[0].color } }}
								class="aspect-auto h-36 w-full"
							>
								<LineChart
									data={row.points}
									x="label"
									{series}
									{padding}
									{annotations}
									{yDomain}
									yBaseline={null}
									points
									props={chartProps}
								>
									{#snippet tooltip()}
										<Chart.Tooltip
											labelFormatter={(v: string) =>
												m.turns_point({
													range: v,
													count: row.points.find((p) => p.label === v)?.games ?? 0
												})}
											valueFormatter={percent}
										/>
									{/snippet}
								</LineChart>
							</Chart.Container>
						{:else}
							<p class="flex h-36 items-center text-sm text-muted-foreground">{m.turns_few()}</p>
						{/if}
					</li>
				{/each}
			</ul>
		</div>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.no_stats()}</p>
	{/if}
</section>
