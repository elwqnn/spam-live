<script lang="ts">
	import { href, percent, winRateColor, type CardMap, type Schemas } from '#lib';
	import { cardPreview } from '#lib/components/CardPreview.svelte';
	import CardThumb from '#lib/components/CardThumb.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';

	let {
		data,
		stats,
		catalog
	}: { data: Schemas['Duo'][]; stats: Schemas['CardStats'][]; catalog: CardMap } = $props();

	const number = new Intl.NumberFormat(getLocale());
	const points = new Intl.NumberFormat(getLocale(), {
		minimumFractionDigits: 1,
		maximumFractionDigits: 1,
		signDisplay: 'exceptZero'
	});
	const rates = $derived(new Map(stats.map((s) => [s.card, s.winRate])));
	const podium = $derived(data.slice(0, 3));
	const ladder = $derived(data.slice(3));
	const ORDER = ['lg:order-2', 'lg:order-1', 'lg:order-3'];
	const STEPS = ['lg:min-h-56', 'lg:min-h-44', 'lg:min-h-36'];

	const name = (id: string) => catalog.get(id)?.name ?? id.slice(0, 8);
	const synergy = (duo: Schemas['Duo']) => {
		const [a, b] = duo.cards.map((id) => rates.get(id));
		if (duo.winRate == null || a == null || b == null) return null;
		const value = Math.round((duo.winRate - (a + b) / 2) * 1000) / 10;
		return {
			sign: Math.sign(value),
			text: m.points({ value: points.format(value) }),
			label:
				value > 0
					? m.duos_synergy_up({ value: points.format(Math.abs(value)) })
					: value < 0
						? m.duos_synergy_down({ value: points.format(Math.abs(value)) })
						: m.duos_synergy_none()
		};
	};
</script>

{#snippet gain(duo: Schemas['Duo'], long: boolean)}
	{@const g = synergy(duo)}
	{#if g}
		<span class="text-xs text-muted-foreground tabular-nums">
			<span aria-hidden="true"
				><span class={['font-medium', g.sign > 0 && 'text-up', g.sign < 0 && 'text-down']}
					>{g.text}</span
				>{long ? ` ${m.duos_synergy()}` : ''}</span
			>
			<span class="sr-only">{g.label}</span>
		</span>
	{/if}
{/snippet}

{#snippet ring(duo: Schemas['Duo'], size: string)}
	<svg viewBox="0 0 36 36" class={['shrink-0 -rotate-90', size]} aria-hidden="true">
		<circle cx="18" cy="18" r="14" fill="none" stroke-width="5" class="stroke-foreground/10" />
		<circle
			cx="18"
			cy="18"
			r="14"
			fill="none"
			stroke-width="5"
			pathLength="100"
			stroke-dasharray="{Math.min(1, Math.max(0, duo.winRate ?? 0)) * 100} 100"
			style:stroke={winRateColor(duo.winRate) ?? 'var(--foreground)'}
		/>
		<line
			x1="0.5"
			x2="7.5"
			y1="18"
			y2="18"
			class="stroke-foreground/50"
			vector-effect="non-scaling-stroke"
		/>
	</svg>
{/snippet}

{#snippet rate(duo: Schemas['Duo'], size: string)}
	<span class={['tracking-tight tabular-nums', size]} style:color={winRateColor(duo.winRate)}
		><span class="sr-only">{m.win_rate()}</span> {percent(duo.winRate)}</span
	>
{/snippet}

{#snippet games(duo: Schemas['Duo'])}
	<span class="text-xs text-muted-foreground tabular-nums"
		>{number.format(duo.games)} {m.games_unit({ count: duo.games })}</span
	>
{/snippet}

{#snippet avatar(id: string, tilt: string)}
	{@const card = catalog.get(id)}
	{#if card}
		<a
			class={[
				'block rounded-xs ring-2 ring-background transition-transform focus-visible:outline-2 focus-visible:outline-offset-2 motion-safe:duration-200',
				tilt
			]}
			href={href(`/cards/${id}`)}
			{@attach cardPreview(card)}
			><img
				src="/art/avatar/{id}.webp"
				alt={card.name ?? id}
				loading="lazy"
				decoding="async"
				class="el-{card.element} size-12 rounded-xs border-2 border-(--el) bg-(--el)"
			/></a
		>
	{:else}
		<span
			class={[
				'grid size-12 place-items-center rounded-xs border border-dashed border-foreground/25 bg-background font-mono text-[10px] text-muted-foreground',
				tilt
			]}><span class="sr-only">{m.unknown_card()}</span> {id.slice(0, 4)}</span
		>
	{/if}
{/snippet}

<section aria-labelledby="duos-title">
	<div class="grid gap-1 section-head">
		<h2 id="duos-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.duos_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.duos_text()}</p>
	</div>
	{#if data.length}
		<ol class="grid gap-x-6 gap-y-12 px-(--gutter) pt-8 lg:grid-cols-3 lg:items-end">
			{#each podium as duo, i (duo.cards.join())}
				{@const [a, b] = duo.cards}
				<li class={['grid gap-8', ORDER[i]]}>
					<div class="group/fan flex justify-center px-6 pt-2">
						{#each [a, b] as id, j (id)}
							<div
								class={[
									'transition-[rotate,translate] motion-safe:duration-300',
									i === 0 ? 'w-36 sm:w-44' : 'w-32 sm:w-36',
									j === 0
										? 'origin-bottom-right -rotate-6 group-hover/fan:-translate-x-2 group-hover/fan:-rotate-9'
										: '-ml-8 origin-bottom-left rotate-6 group-hover/fan:translate-x-2 group-hover/fan:rotate-9'
								]}
							>
								<CardThumb {id} card={catalog.get(id)} large={i === 0} />
							</div>
						{/each}
					</div>
					<div class={['flex flex-col bg-card', STEPS[i]]}>
						<span class="flex h-1" aria-hidden="true">
							{#each [a, b] as id (id)}
								<span class="el-{catalog.get(id)?.element} flex-1 bg-[var(--el,var(--border))]"
								></span>
							{/each}
						</span>
						<div class="grid flex-1 content-start gap-3 border-x border-b px-5 py-4">
							<div class="flex items-start justify-between gap-4">
								<span
									class={['font-heading leading-none', i === 0 ? 'text-6xl' : 'text-5xl']}
									aria-hidden="true">{i + 1}</span
								>
								<div class="grid min-w-0 justify-items-end text-right text-sm">
									<span class="max-w-full truncate">{name(a)}</span>
									<span class="max-w-full truncate">{name(b)}</span>
								</div>
							</div>
							<div class="flex items-center gap-4">
								{@render ring(duo, i === 0 ? 'size-16' : 'size-14')}
								<p class="grid gap-1">
									{@render rate(duo, i === 0 ? 'text-4xl' : 'text-3xl')}
									{@render games(duo)}
								</p>
							</div>
							{@render gain(duo, true)}
						</div>
					</div>
				</li>
			{/each}
		</ol>
		{#if ladder.length}
			<ol class="mt-12 divide-y border-t" start="4">
				{#each ladder as duo, i (duo.cards.join())}
					{@const [a, b] = duo.cards}
					<li
						class="group/fan grid grid-cols-[2rem_auto_minmax(0,1fr)] items-center gap-x-4 gap-y-3 px-(--gutter) py-3 transition-colors hover:bg-row sm:grid-cols-[2.5rem_auto_minmax(0,1fr)_16rem]"
					>
						<span class="font-heading text-xl text-muted-foreground tabular-nums" aria-hidden="true"
							>{i + 4}</span
						>
						<div class="flex py-1">
							{@render avatar(
								a,
								'relative -rotate-6 group-hover/fan:-translate-x-0.5 group-hover/fan:-rotate-9'
							)}
							{@render avatar(
								b,
								'-ml-3 rotate-6 group-hover/fan:translate-x-0.5 group-hover/fan:rotate-9'
							)}
						</div>
						<div class="grid min-w-0 text-sm">
							<span class="truncate">{name(a)}</span>
							<span class="truncate">{name(b)}</span>
						</div>
						<div
							class="col-span-2 col-start-2 flex items-center gap-3 sm:col-span-1 sm:col-start-auto"
						>
							{@render ring(duo, 'size-10')}
							<div class="grid min-w-0 flex-1 gap-0.5">
								<p class="flex flex-wrap items-baseline justify-between gap-x-3">
									{@render rate(duo, 'text-lg')}
									{@render gain(duo, false)}
								</p>
								{@render games(duo)}
							</div>
						</div>
					</li>
				{/each}
			</ol>
		{/if}
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.no_stats()}</p>
	{/if}
</section>
