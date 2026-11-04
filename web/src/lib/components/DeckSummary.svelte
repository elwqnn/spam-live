<script lang="ts">
	import { href, percent, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import CardBadge from './CardBadge.svelte';
	import CardBadgeStack from './CardBadgeStack.svelte';

	let { deck, cards }: { deck: Schemas['DeckDetail']; cards: (Schemas['CardBase'] | null)[] } =
		$props();

	const known = $derived(cards.filter((c) => c !== null));
	const hp = $derived(known.reduce((sum, c) => sum + c.hp, 0));
	const income = $derived(known.reduce((sum, c) => sum + c.goldIncome, 0));
	const mix = $derived(
		known.reduce(
			(map, c) => map.set(c.element, (map.get(c.element) ?? 0) + 1),
			new Map<string, number>()
		)
	);
	const stats = $derived(deck.stats.current);
	const num = (n: number) => n.toLocaleString(getLocale(), { maximumFractionDigits: 1 });
	const metrics = $derived([
		[m.cards_title(), deck.cards.length],
		[m.total_hp(), hp],
		[m.total_income(), income],
		[m.average_hp(), known.length ? hp / known.length : 0]
	] as const);
</script>

<section class="grid md:grid-cols-[minmax(0,16rem)_minmax(0,1fr)]" aria-label={m.deck_summary()}>
	<div class="grid content-start gap-1 border-b px-4 py-6 sm:px-6 md:border-r md:border-b-0">
		<span class="eyebrow text-muted-foreground"
			>{m.win_rate()} · {m.version_label({ version: deck.stats.version })}</span
		>
		<span class="text-5xl font-medium tracking-tight tabular-nums">{percent(stats.winRate)}</span>
		<span class="text-sm text-muted-foreground tabular-nums">{m.games()} {stats.games}</span>
	</div>
	<div class="grid min-w-0 content-start">
		<dl class="cells grid grid-cols-2 sm:grid-cols-4">
			{#each metrics as [label, value] (label)}
				<div class="grid gap-1 px-4 py-4 sm:px-6">
					<dt class="eyebrow text-[10px] text-muted-foreground">{label}</dt>
					<dd class="text-xl tabular-nums">{num(value)}</dd>
				</div>
			{/each}
			<div class="col-span-full grid gap-1.5 px-4 py-4 sm:px-6">
				<dt class="eyebrow text-[10px] text-muted-foreground">{m.element_mix()}</dt>
				<dd>
					<CardBadgeStack>
						{#each mix as [element, count] (element)}
							<CardBadge type="element" value={element}
								><span class="tabular-nums">×{count}</span></CardBadge
							>
						{/each}
					</CardBadgeStack>
				</dd>
			</div>
		</dl>
		<ul class="flex flex-wrap gap-2 border-t px-4 py-4 sm:px-6" aria-label={m.cards_title()}>
			{#each known as card (card.id)}
				<li>
					<a
						href={href(`/cards/${card.id}`)}
						title={card.name ?? card.id}
						class="group block rounded-xs"
					>
						<img
							src="/art/avatar/{card.id}.webp"
							alt=""
							loading="lazy"
							decoding="async"
							class="el-{card.element} size-11 rounded-xs border-2 border-(--el) bg-(--el) transition-colors group-hover:border-foreground"
						/>
						<span class="sr-only">{card.name ?? card.id}</span>
					</a>
				</li>
			{/each}
		</ul>
	</div>
</section>
