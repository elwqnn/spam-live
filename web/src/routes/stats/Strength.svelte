<script lang="ts">
	import { href, percent, winRateColor, type CardMap, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import ElementLegend from './ElementLegend.svelte';
	import { squarify, type Box } from './treemap.js';

	let { data, catalog }: { data: Schemas['CardStats'][]; catalog: CardMap } = $props();

	const number = new Intl.NumberFormat(getLocale());
	let width = $state(1200);
	let height = $state(640);

	const items = $derived(
		data.flatMap((s) => {
			const card = catalog.get(s.card);
			return card && s.playRate > 0
				? [{ ...s, name: card.name ?? s.card.slice(0, 8), element: card.element }]
				: [];
		})
	);
	type Item = (typeof items)[number];

	const inset = (b: Box, d: number) => ({ x: b.x + d, y: b.y + d, w: b.w - 2 * d, h: b.h - 2 * d });
	const tiles = $derived(
		squarify(items, (s) => s.playRate, { x: 0, y: 0, w: width, h: height }).map((t) => ({
			...inset(t, 1),
			s: t.item
		}))
	);
	const place = (b: Box) =>
		`left:${(b.x / width) * 100}%;top:${(b.y / height) * 100}%;width:${(Math.max(0, b.w) / width) * 100}%;height:${(Math.max(0, b.h) / height) * 100}%`;

	let active = $state.raw<Item | null>(null);
	const hide = (s: Item) => {
		if (active === s) active = null;
	};
</script>

<section aria-labelledby="strength-title">
	<div class="grid gap-1 section-head">
		<h2 id="strength-title" class="font-heading text-2xl font-normal tracking-tight">
			{m.strength_title()}
		</h2>
		<p class="text-sm text-muted-foreground">{m.strength_text()}</p>
	</div>
	{#if items.length}
		<ElementLegend />
		<figure class="grid gap-3 px-(--gutter) py-4">
			<div
				class="flex min-h-12 items-center gap-3 text-sm pointer-coarse:hidden"
				aria-hidden="true"
			>
				{#if active}
					<img
						src="/art/avatar/{active.card}.webp"
						alt=""
						decoding="async"
						class="el-{active.element} size-10 shrink-0 rounded-xs border-2 border-(--el) bg-(--el)"
					/>
					<span class="min-w-0 truncate font-medium">{active.name}</span>
					<dl class="ml-auto flex shrink-0 gap-4 text-xs tabular-nums sm:gap-6">
						{#each [[m.play_rate(), percent(active.playRate), undefined], [m.win_rate(), percent(active.winRate), winRateColor(active.winRate)], [m.games(), number.format(active.games), undefined]] as [label, value, color] (label)}
							<div class="grid gap-0.5">
								<dt class="text-muted-foreground">{label}</dt>
								<dd class="text-sm font-medium" style:color>{value}</dd>
							</div>
						{/each}
					</dl>
				{:else}
					<span class="text-muted-foreground">{m.strength_hint()}</span>
				{/if}
			</div>
			<div
				role="group"
				aria-label={m.strength_label()}
				class="relative h-[36rem] overflow-hidden sm:h-[40rem]"
				bind:clientWidth={width}
				bind:clientHeight={height}
			>
				{#each tiles as { s, ...t } (s.card)}
					<a
						href={href(`/cards/${s.card}`)}
						class="el-{s.element} group absolute overflow-hidden rounded-xs border-2 border-(--el) bg-(--el) hover:z-1 hover:outline-2 hover:outline-foreground focus-visible:z-1 focus-visible:outline-2 focus-visible:outline-foreground"
						style={place(t)}
						aria-label={m.strength_tile({
							name: s.name,
							playRate: percent(s.playRate),
							winRate: percent(s.winRate)
						})}
						onpointerenter={() => (active = s)}
						onpointerleave={() => hide(s)}
						onfocus={() => (active = s)}
						onblur={() => hide(s)}
					>
						<img
							src="/art/avatar/{s.card}.webp"
							alt=""
							loading="lazy"
							decoding="async"
							class="size-full object-cover"
						/>
						{#if t.w >= 96 && t.h >= 72}
							<span
								class="absolute inset-x-0 bottom-0 grid bg-background/85 px-2 py-1 text-xs tabular-nums"
								aria-hidden="true"
							>
								<span class="truncate font-medium">{s.name}</span>
								<span class="text-muted-foreground">{percent(s.playRate)}</span>
							</span>
						{/if}
					</a>
				{/each}
			</div>
		</figure>
	{:else}
		<p class="px-(--gutter) pb-8 text-sm text-muted-foreground">{m.no_stats()}</p>
	{/if}
</section>
