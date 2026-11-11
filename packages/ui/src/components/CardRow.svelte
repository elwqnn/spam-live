<script lang="ts">
	import type { Snippet } from 'svelte';
	import { percent, type Schemas } from '#ui';
	import { art, href } from '$host';
	import { m } from '$paraglide/messages.js';
	import CardBadge from './CardBadge.svelte';
	import CardBadgeStack from './CardBadgeStack.svelte';
	import Change from './Change.svelte';
	import TierBadge from './TierBadge.svelte';
	import WinRate from './WinRate.svelte';

	let {
		card,
		stats,
		active = false,
		chart,
		details,
		action
	}: {
		card: Schemas['CardBase'];
		stats?: Schemas['CardStats'];
		active?: boolean;
		chart?: Snippet;
		details?: Snippet;
		action?: Snippet;
	} = $props();
</script>

<div
	class={[
		'relative flex items-center gap-3 px-(--gutter) py-3 transition-colors hover:bg-row sm:gap-4',
		active && 'bg-row'
	]}
	aria-current={active ? 'true' : undefined}
>
	<img
		src={art(`avatar/${card.id}.webp`)}
		alt=""
		loading="lazy"
		decoding="async"
		class="el-{card.element} size-10 shrink-0 rounded-xs border-2 border-(--el) bg-(--el)"
	/>
	<div class="grid min-w-0 flex-1 gap-1.5">
		<a
			class="text-[15px] font-medium wrap-anywhere after:absolute after:inset-0 hover:underline"
			href={href(`/cards/${card.id}`)}>{card.name ?? card.id}</a
		>
		<CardBadgeStack class="relative">
			<CardBadge type="element" value={card.element} display="compact" />
			<CardBadge type="rarity" value={card.rarity} display="compact" />
			<CardBadge type="hp" value={card.hp} />
			<CardBadge type="income" value={card.goldIncome} />
			{#if stats?.tier}<TierBadge tier={stats.tier} />{/if}
		</CardBadgeStack>
		{#if details}<div class="relative">{@render details()}</div>{/if}
	</div>
	{#if chart}<div class="relative hidden sm:block">{@render chart()}</div>{/if}
	{#if stats}
		<dl class="flex shrink-0 items-center gap-x-5 text-[13px] tabular-nums">
			<div class="hidden gap-0.5 text-right md:grid">
				<dt class="eyebrow text-[10px] text-muted-foreground">{m.play_rate()}</dt>
				<dd class="grid justify-items-end">
					{percent(stats.playRate)}
					<Change value={stats.playRateChange} class="text-[11px]" />
				</dd>
			</div>
			<div class="grid gap-0.5 text-right">
				<dt class="eyebrow text-[10px] text-muted-foreground">{m.games()}</dt>
				<dd>{stats.games}</dd>
			</div>
			<div class="grid gap-0.5">
				<dt class="text-right eyebrow text-[10px] text-muted-foreground">{m.win_rate()}</dt>
				<dd class="grid justify-items-end">
					<WinRate value={stats.winRate} element={card.element} />
					<Change value={stats.winRateChange} color class="text-[11px]" />
				</dd>
			</div>
		</dl>
	{/if}
	{#if action}<div class="relative">{@render action()}</div>{/if}
</div>
