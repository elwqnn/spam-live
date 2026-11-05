<script lang="ts">
	import { href, percent, winRateColor, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import CardArt from './CardArt.svelte';
	import Change from './Change.svelte';

	let {
		card,
		stats
	}: {
		card: Schemas['CardBase'];
		stats?: Pick<
			Schemas['CardStats'],
			'games' | 'playRate' | 'winRate' | 'winRateChange' | 'playRateChange'
		>;
	} = $props();

	const id = $props.id();
	const name = $derived(card.name ?? card.id);
</script>

<a
	href={href(`/cards/${card.id}`)}
	aria-label={name}
	aria-describedby={stats ? id : undefined}
	class={[
		'flex h-full flex-col transition-colors outline-none [content-visibility:auto] [overflow-clip-margin:16px] hover:bg-row focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset',
		stats ? '[contain-intrinsic-size:auto_400px]' : '[contain-intrinsic-size:auto_360px]'
	]}
>
	<div class="p-2 pb-3 sm:p-3 sm:pb-4"><CardArt {card} interactive /></div>
	{#if stats}
		<div
			{id}
			class="flex flex-col gap-1 border-t px-3 py-2.5 tabular-nums in-[.stats-off]:hidden sm:px-4 sm:py-3"
		>
			<span class="flex items-baseline justify-between gap-3">
				<span class="min-w-0 truncate text-sm font-medium" aria-hidden="true">{name}</span>
				<span class={['shrink-0 text-[17px] leading-none font-medium', winRateColor(stats.winRate)]}
					><span class="sr-only">{m.win_rate()}</span> {percent(stats.winRate)}</span
				>
			</span>
			<span class="flex flex-wrap items-baseline justify-between gap-x-3">
				<span class="eyebrow text-muted-foreground"
					>{m.play_rate()} {percent(stats.games ? stats.playRate : null)}</span
				>
				<Change value={stats.playRateChange} label={m.play_rate()} class="text-xs" />
			</span>
			<span class="flex flex-wrap items-baseline justify-between gap-x-3">
				<span class="eyebrow text-muted-foreground"
					>{stats.games} {m.games_unit({ count: stats.games })}</span
				>
				<Change value={stats.winRateChange} label={m.win_rate()} color class="text-xs" />
			</span>
		</div>
	{/if}
</a>
