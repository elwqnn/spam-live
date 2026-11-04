<script lang="ts">
	import { href, percent, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import CardArt from './CardArt.svelte';
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

<section
	class="grid gap-6 rounded-xl border bg-card p-5 sm:p-6 md:grid-cols-[minmax(0,14rem)_minmax(0,1fr)] md:gap-8"
	aria-label={m.deck_summary()}
>
	<div class="grid content-start gap-1">
		<span class="eyebrow text-muted-foreground"
			>{m.win_rate()} · {m.version_label({ version: deck.stats.version })}</span
		>
		<span class="text-5xl font-medium tracking-tight tabular-nums">{percent(stats.winRate)}</span>
		<span class="text-sm text-muted-foreground tabular-nums">{m.games()} {stats.games}</span>
	</div>
	<div class="grid content-start gap-5">
		<dl class="grid grid-cols-2 gap-x-6 gap-y-4 sm:grid-cols-4">
			{#each metrics as [label, value] (label)}
				<div class="grid gap-1">
					<dt class="eyebrow text-[10px] text-muted-foreground">{label}</dt>
					<dd class="text-xl tabular-nums">{num(value)}</dd>
				</div>
			{/each}
			<div class="col-span-full grid gap-1.5">
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
		<ul class="flex flex-wrap gap-2" aria-label={m.cards_title()}>
			{#each known as card (card.id)}
				<li>
					<a
						href={href(`/cards/${card.id}`)}
						title={card.name ?? card.id}
						class="block rounded-xs spring motion-safe:hover:-translate-y-0.5"
					>
						<CardArt
							element={card.element}
							rarity={card.rarity}
							tag={false}
							class="aspect-5/7 w-11 rounded-xs"
						/>
						<span class="sr-only">{card.name ?? card.id}</span>
					</a>
				</li>
			{/each}
		</ul>
	</div>
</section>
