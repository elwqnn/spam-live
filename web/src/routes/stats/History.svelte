<script lang="ts">
	import { formatChange, percent, winRateColor, type Schemas } from '#lib';
	import { ELEMENTS } from '#lib/components/CardBadge.svelte';
	import Change from '#lib/components/Change.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import Sparkline from './Sparkline.svelte';
	import { fullWeeks, isoDay, monday, spanRange } from './spark.js';

	let { data }: { data: Schemas['HistoryStats'] } = $props();

	const WEEKS = 12;
	const day = new Intl.DateTimeFormat(getLocale(), {
		day: 'numeric',
		month: 'short',
		timeZone: 'UTC'
	});
	const date = (week: string) => day.format(new Date(`${week}T00:00:00Z`));

	const span = $derived(
		fullWeeks(
			data.weeks.map((w) => w.week),
			new Date(),
			WEEKS
		)
	);
	const table = $derived.by(() => {
		if (!span) return [];
		const { recent } = span;
		const totals = recent.map((week) =>
			data.weeks.filter((w) => w.week === week).reduce((n, w) => n + w.games, 0)
		);
		const starts = new Set(data.versions.map((v) => isoDay(monday(new Date(v.start)))));
		const ticks = recent.flatMap((week, i) => (starts.has(week) ? [i] : []));
		return Object.keys(ELEMENTS)
			.map((key) => {
				const items = recent.map((week) =>
					data.weeks.find((w) => w.week === week && w.element === key)
				);
				const share = items.map((w, i) => (totals[i] ? (w?.games ?? 0) / totals[i] : null));
				const rate = items.map((w) => w?.winRate ?? null);
				const at = recent.length - 1;
				const now = items[at];
				const before = items[at - 1];
				return {
					key,
					share,
					rate,
					ticks,
					current: now ? share[at] : null,
					winRate: now?.winRate ?? null,
					shareChange: now && totals[at - 1] ? share[at]! - share[at - 1]! : null,
					rateChange: now && before ? now.winRate - before.winRate : null
				};
			})
			.sort((a, b) => (b.current ?? -1) - (a.current ?? -1));
	});

	const against = (value: number | null) => {
		const change = formatChange(value, getLocale());
		if (!change) return '';
		const against = m.history_against();
		return change.sign > 0
			? m.change_up({ value: change.value, against })
			: change.sign < 0
				? m.change_down({ value: change.value, against })
				: m.change_none({ against });
	};
</script>

<section aria-labelledby="history-title">
	<div class="grid gap-1 section-head">
		<h2 id="history-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.history_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.history_text()}</p>
	</div>
	{#if !span}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.history_few()}</p>
	{:else}
		<div>
			<table class="w-full text-sm tabular-nums">
				<caption class="sr-only">{m.history_caption()}</caption>
				<thead>
					<tr class="text-left eyebrow text-[10px] text-muted-foreground">
						<th scope="col" class="py-2 pl-(--gutter) font-medium">{m.history_element()}</th>
						<th scope="col" class="px-2 py-2 font-medium">{m.history_share()}</th>
						<th scope="col" class="py-2 pr-(--gutter) pl-2 font-medium">{m.win_rate()}</th>
					</tr>
				</thead>
				<tbody>
					{#each table as row (row.key)}
						<tr class="border-t">
							<th scope="row" class="py-3 pl-(--gutter) text-left font-normal">
								<span class="flex items-center gap-2">
									<img
										src="/art/ui/element_{row.key}.webp"
										alt=""
										aria-hidden="true"
										decoding="async"
										class="size-5 shrink-0"
									/>
									<span class="whitespace-nowrap">{ELEMENTS[row.key]()}</span>
								</span>
							</th>
							<td class="px-2 py-3">
								<div class="flex flex-wrap items-center gap-x-4 gap-y-1.5">
									<span class="grid min-w-16">
										<span>{percent(row.current)}</span>
										{#if row.shareChange != null}
											<span aria-hidden="true"
												><Change value={row.shareChange} class="text-xs" /></span
											>
											<span class="sr-only">{against(row.shareChange)}</span>
										{/if}
									</span>
									<Sparkline
										values={row.share}
										range={spanRange(row.share, 0.02)}
										ticks={row.ticks}
										width={160}
										height={32}
										class="h-6 w-24 lg:h-8 lg:w-40"
										color="var(--el-{row.key})"
									/>
								</div>
							</td>
							<td class="py-3 pr-(--gutter) pl-2">
								<div class="flex flex-wrap items-center gap-x-4 gap-y-1.5">
									<span class="grid min-w-16">
										<span style:color={winRateColor(row.winRate)}>{percent(row.winRate)}</span>
										{#if row.rateChange != null}
											<span aria-hidden="true"
												><Change value={row.rateChange} color class="text-xs" /></span
											>
											<span class="sr-only">{against(row.rateChange)}</span>
										{/if}
									</span>
									<Sparkline
										values={row.rate}
										range={spanRange(row.rate, 0.04)}
										mid={0.5}
										ticks={row.ticks}
										width={160}
										height={32}
										class="h-6 w-24 lg:h-8 lg:w-40"
										color="var(--el-{row.key})"
									/>
								</div>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<p class="border-t px-(--gutter) py-3 text-sm text-muted-foreground">
			{m.history_compare({ last: date(span.last), before: date(span.before) })}
		</p>
	{/if}
</section>
