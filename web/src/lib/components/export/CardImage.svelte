<script lang="ts">
	import { percent, winRateColor, type CardDetail, type Schemas } from '#lib';
	import CardArt from '@spam-live/ui/components/CardArt.svelte';
	import TierBadge from '@spam-live/ui/components/TierBadge.svelte';
	import { BADGE, type Size } from '#lib/export.js';
	import { m } from '#lib/paraglide/messages.js';
	import Stat from './Stat.svelte';

	let { card, stats, size }: { card: CardDetail; stats: Schemas['CardStats'] | null; size: Size } =
		$props();

	const WIDTH = { wide: 600, square: 460, vertical: 680 };
</script>

<div
	class={[
		'flex h-full items-center gap-[64px]',
		size.id === 'vertical' && 'flex-col justify-center'
	]}
>
	<div class="shrink-0" style:width="{WIDTH[size.id]}px"><CardArt {card} /></div>
	<div class="grid min-w-0 gap-[48px]">
		<div
			class={[
				'flex',
				size.id === 'square' ? 'flex-col items-start gap-[24px]' : 'items-center gap-[32px]'
			]}
		>
			{#if stats?.tier}
				<div class="flex shrink-0" style:width="{BADGE}px" style:height="{BADGE}px">
					<TierBadge tier={stats.tier} class="size-full rounded-sm text-[48px]" />
				</div>
			{/if}
			<h2
				class={[
					'font-heading leading-[1.05] font-normal tracking-tight text-balance wrap-anywhere',
					size.id === 'square' ? 'text-[64px]' : 'text-[88px]'
				]}
			>
				{card.name ?? card.id}
			</h2>
		</div>
		<dl
			class={[
				'grid gap-x-[48px] gap-y-[32px]',
				size.id === 'square' ? 'grid-cols-1' : 'grid-cols-3'
			]}
		>
			<Stat
				label={m.win_rate()}
				value={percent(stats?.winRate)}
				color={winRateColor(stats?.winRate)}
			/>
			<Stat label={m.play_rate()} value={percent(stats?.playRate)} />
			<Stat label={m.games()} value={stats?.games ?? 0} />
		</dl>
	</div>
</div>
