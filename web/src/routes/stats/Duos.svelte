<script lang="ts">
	import { percent, winRateColor, type CardMap, type Schemas } from '#lib';
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

	const synergy = (duo: Schemas['Duo']) => {
		const [a, b] = duo.cards.map((id) => rates.get(id));
		if (duo.winRate == null || a == null || b == null) return null;
		const value = Math.round((duo.winRate - (a + b) / 2) * 1000) / 10;
		const shown = points.format(value);
		return {
			sign: Math.sign(value),
			text: m.points({ value: shown }),
			label:
				value > 0
					? m.duos_synergy_up({ value: points.format(Math.abs(value)) })
					: value < 0
						? m.duos_synergy_down({ value: points.format(Math.abs(value)) })
						: m.duos_synergy_none()
		};
	};
</script>

<section aria-labelledby="duos-title">
	<div class="grid gap-1 section-head">
		<h2 id="duos-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.duos_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.duos_text()}</p>
	</div>
	{#if data.length}
		<div>
			<ol class="cells grid sm:grid-cols-2 lg:grid-cols-3">
				{#each data as duo, i (duo.cards.join())}
					{@const large = i === 0}
					{@const gain = synergy(duo)}
					<li
						class={[
							'flex flex-col gap-4 px-(--gutter) py-4 sm:py-5',
							large && 'lg:col-span-2 lg:row-span-2'
						]}
					>
						<span class="text-sm text-muted-foreground tabular-nums"
							>#{String(i + 1).padStart(2, '0')}</span
						>
						<ul
							class={[
								'grid grid-cols-2 gap-3 pt-[4.5%] pl-[4.5%]',
								large ? 'max-w-64 lg:max-w-md' : 'max-w-64'
							]}
						>
							{#each duo.cards as id (id)}
								<li class="grid min-w-0 content-start gap-2">
									<CardThumb {id} card={catalog.get(id)} {large} />
									<span
										class={['truncate text-sm', !catalog.has(id) && 'font-mono text-xs']}
										aria-hidden="true">{catalog.get(id)?.name ?? id.slice(0, 8)}</span
									>
								</li>
							{/each}
						</ul>
						<div class="mt-auto grid gap-2">
							<p class="flex flex-wrap items-baseline justify-between gap-x-3 tabular-nums">
								<span
									class={large
										? 'shader-text text-4xl font-medium tracking-tight lg:text-5xl'
										: 'text-2xl tracking-tight'}
									style:color={large ? undefined : winRateColor(duo.winRate)}
									><span class="sr-only">{m.win_rate()}</span> {percent(duo.winRate)}</span
								>
								<span class="text-xs text-muted-foreground"
									>{number.format(duo.games)} {m.games_unit({ count: duo.games })}</span
								>
							</p>
							<span
								class={['relative block rounded-[2px] bg-foreground/6', large ? 'h-3' : 'h-2']}
								aria-hidden="true"
							>
								<span
									class="block h-full origin-left rounded-[2px] motion-safe:animate-grow"
									style:width="{Math.min(1, Math.max(0, duo.winRate ?? 0)) * 100}%"
									style:background-color={winRateColor(duo.winRate) ?? 'var(--foreground)'}
								></span>
								<span class="absolute -inset-y-1 left-1/2 w-px bg-foreground/50"></span>
							</span>
							{#if gain}
								<p class="text-xs text-muted-foreground tabular-nums">
									<span aria-hidden="true"
										><span
											class={[
												'font-medium',
												gain.sign > 0 && 'text-up',
												gain.sign < 0 && 'text-down'
											]}>{gain.text}</span
										>
										{m.duos_synergy()}</span
									>
									<span class="sr-only">{gain.label}</span>
								</p>
							{/if}
						</div>
					</li>
				{/each}
			</ol>
		</div>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.no_stats()}</p>
	{/if}
</section>
