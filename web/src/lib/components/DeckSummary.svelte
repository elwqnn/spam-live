<script lang="ts">
	import { percent, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';

	let { deck, cards }: { deck: Schemas['DeckDetail']; cards: (Schemas['CardBase'] | null)[] } =
		$props();

	const known = $derived(cards.filter((c) => c !== null));
	const hp = $derived(known.reduce((sum, c) => sum + c.hp, 0));
	const income = $derived(known.reduce((sum, c) => sum + c.goldIncome, 0));
	const stats = $derived(deck.stats.current);
	const num = (n: number) => n.toLocaleString(getLocale(), { maximumFractionDigits: 1 });
	const metrics = $derived([
		[m.total_hp(), hp],
		[m.total_income(), income]
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
		<dl class="cells grid grid-cols-2">
			{#each metrics as [label, value] (label)}
				<div class="grid gap-1 px-4 py-4 sm:px-6">
					<dt class="eyebrow text-[10px] text-muted-foreground">{label}</dt>
					<dd class="text-xl tabular-nums">{num(value)}</dd>
				</div>
			{/each}
		</dl>
	</div>
</section>
