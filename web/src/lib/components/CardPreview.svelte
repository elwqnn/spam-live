<script lang="ts" module>
	import type { Attachment } from 'svelte/attachments';
	import type { Schemas } from '#lib';
	import type { Point, Rect } from '#lib/place.js';

	const preview = $state<{
		card: Schemas['CardBase'] | null;
		mode: 'pointer' | 'focus';
		at: Point | Rect;
		owner: HTMLElement | null;
	}>({ card: null, mode: 'pointer', at: { x: 0, y: 0 }, owner: null });

	const fine = () => matchMedia('(hover: hover) and (pointer: fine)').matches;

	function close() {
		preview.card = null;
		preview.owner = null;
	}

	export const cardPreview =
		(card: Schemas['CardBase']): Attachment<HTMLElement> =>
		(node) => {
			const owns = (mode: 'pointer' | 'focus') => preview.owner === node && preview.mode === mode;
			const enter = (e: PointerEvent) => {
				if (e.pointerType === 'mouse' && fine())
					Object.assign(preview, {
						card,
						mode: 'pointer',
						at: { x: e.clientX, y: e.clientY },
						owner: node
					});
			};
			const move = (e: PointerEvent) => {
				if (owns('pointer')) preview.at = { x: e.clientX, y: e.clientY };
			};
			const leave = () => {
				if (owns('pointer')) close();
			};
			const focus = () => {
				if (node.matches(':focus-visible') && fine())
					Object.assign(preview, {
						card,
						mode: 'focus',
						at: node.getBoundingClientRect(),
						owner: node
					});
			};
			const blur = () => {
				if (owns('focus')) close();
			};
			const events = { pointerenter: enter, pointermove: move, pointerleave: leave, focus, blur };
			for (const [type, listener] of Object.entries(events))
				node.addEventListener(type, listener as EventListener);
			return () => {
				for (const [type, listener] of Object.entries(events))
					node.removeEventListener(type, listener as EventListener);
				if (preview.owner === node) close();
			};
		};
</script>

<script lang="ts">
	import { page } from '$app/state';
	import { percent, winRateColor } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import { place } from '#lib/place.js';
	import CardBadge from './CardBadge.svelte';
	import CardBadgeStack from './CardBadgeStack.svelte';
	import RarityMark from './RarityMark.svelte';
	import ToxIcon from './ToxIcon.svelte';

	let popup = $state<HTMLElement>();
	const card = $derived(preview.card);
	const stats = $derived.by(() => {
		const list: Schemas['CardStats'][] | undefined = page.data.cardStats;
		if (!card || !Array.isArray(list)) return null;
		return list.find((s) => s.card === card.id) ?? { games: 0, playRate: 0, winRate: null };
	});

	$effect(() => {
		if (!popup) return;
		popup.togglePopover(!!card);
		if (!card) return;
		const { left, top } = place(
			preview.at,
			{ width: popup.offsetWidth, height: popup.offsetHeight },
			{
				width: document.documentElement.clientWidth,
				height: document.documentElement.clientHeight
			}
		);
		popup.style.left = `${left}px`;
		popup.style.top = `${top}px`;
	});

	$effect(() => {
		if (!card) return;
		const key = (e: KeyboardEvent) => {
			if (e.key === 'Escape') close();
		};
		const replace = () => {
			preview.at =
				preview.mode === 'focus' && preview.owner
					? preview.owner.getBoundingClientRect()
					: { ...preview.at };
		};
		document.addEventListener('keydown', key);
		document.addEventListener('scroll', replace, { capture: true, passive: true });
		addEventListener('resize', replace);
		return () => {
			document.removeEventListener('keydown', key);
			document.removeEventListener('scroll', replace, { capture: true });
			removeEventListener('resize', replace);
		};
	});
</script>

{#snippet stat(label: string, value: string | number, tone?: string)}
	<div class="grid gap-0.5">
		<dt class="eyebrow text-muted-foreground">{label}</dt>
		<dd class={['tabular-nums', tone]}>{value}</dd>
	</div>
{/snippet}

<div
	bind:this={popup}
	popover="manual"
	aria-hidden="true"
	class="pointer-events-none inset-auto m-0 w-72 divide-y rounded-sm border bg-popover p-0 text-sm text-popover-foreground"
>
	{#if card}
		<div class="flex gap-3 p-3">
			<span
				class="el-{card.element} aspect-[1432/2048] w-14 shrink-0 bg-[color-mix(in_oklab,var(--el)_40%,#000)] [mask:url(/art/ui/card_mask.webp)_0_0/100%_100%]"
			>
				<img
					src="/art/thumb/{card.id}.webp"
					alt=""
					decoding="async"
					class="size-full object-cover"
				/>
			</span>
			<div class="grid min-w-0 content-start gap-2">
				<p class="leading-snug font-medium wrap-anywhere">{card.name ?? card.id}</p>
				<CardBadgeStack>
					<CardBadge type="element" value={card.element} display="compact" />
					<CardBadge type="rarity" value={card.rarity} display="compact" />
					<CardBadge type="hp" value={card.hp} />
					<CardBadge type="income" value={card.goldIncome} display="compact" />
				</CardBadgeStack>
			</div>
		</div>
		{#if card.face.abilities.length}
			<div class="grid gap-1.5 p-3">
				<p class="eyebrow text-muted-foreground">{m.abilities()}</p>
				<ul class="grid gap-1">
					{#each card.face.abilities as ability, i (i)}
						<li class="flex items-center justify-between gap-2">
							<span class="min-w-0 truncate">{ability.name}</span>
							<CardBadge type="cooldown" value={ability.cooldown} display="compact" />
						</li>
					{/each}
				</ul>
			</div>
		{/if}
		{#if card.face.tox.length}
			<div class="grid gap-1.5 p-3">
				<p class="eyebrow text-muted-foreground">{m.tox()}</p>
				<ul class="grid gap-1">
					{#each card.face.tox as tox, i (i)}
						<li class="flex items-center justify-between gap-2">
							<ToxIcon id={tox.id} image={tox.image} lazy={false} class="size-5" />
							<span class="mr-auto min-w-0 truncate">{tox.name}</span>
							{#if tox.rarity}<RarityMark rarity={tox.rarity} tooltip={false} />{/if}
						</li>
					{/each}
				</ul>
			</div>
		{/if}
		{#if stats}
			<dl class="grid grid-cols-3 gap-2 p-3">
				{@render stat(m.play_rate(), percent(stats.games ? stats.playRate : null))}
				{@render stat(m.games(), stats.games)}
				{@render stat(m.win_rate(), percent(stats.winRate), winRateColor(stats.winRate))}
			</dl>
		{/if}
	{/if}
</div>
