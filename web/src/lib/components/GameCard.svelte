<script lang="ts" module>
	const ROWS = [55.381, 66.479, 77.48];
	const tier = (rarity: string | null | undefined) => rarity?.replace('rarity-', '');
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import { cn } from '#lib/utils.js';

	let {
		card,
		class: className,
		children
	}: { card: Schemas['CardBase']; class?: string; children?: Snippet } = $props();

	const name = $derived(card.name ?? card.id);
	const rows = $derived(
		[
			...card.face.abilities.map((a) => ({ ...a, tox: undefined })),
			...card.face.tox.map((t) => ({ ...t, tox: tier(t.rarity) ?? '' }))
		].slice(0, 3)
	);
</script>

<div class={cn('card', className)} data-element={card.element}>
	<div class="rim"></div>
	<div class="masked">
		<img class="art" src="/art/card/{card.id}.webp" alt="" loading="lazy" decoding="async" />
		<div class="header"></div>
		<span class="text name" style:--n={name.length}>{name}</span>
		<img class="fill" src="/art/ui/card_front_{card.element}.webp" alt="" />
		{@render children?.()}
	</div>
	<img class="badge" src="/art/ui/element_{card.element}.webp" alt="" />
	<img class="gem" src="/art/ui/card_rarity_{tier(card.rarity)}.webp" alt="" />
	{#each rows as row, i (i)}
		<div class="slot" style:top="{ROWS[i]}%">
			{#if row.tox === undefined}
				<div class="ability-bg"></div>
				{#if row.element}
					<span class="ability-element" data-element={row.element}>
						<span class="aura"></span>
						<img src="/art/ui/element_{row.element}.webp" alt="" />
					</span>
				{/if}
				<span class="text ability-name" style:--n={row.name.length}>{row.name}</span>
				<span class="cooldown"></span>
				<span class="text cooldown-amount"
					><span aria-hidden="true">{row.cooldown}</span><span class="sr-only"
						>{m.cooldown({ turns: row.cooldown })}</span
					></span
				>
			{:else}
				<span class="tox-aura" data-rarity={row.tox}></span>
				<span class="tox">
					{#if row.tox}<img
							src="/art/ui/card_ability_passive_line_{row.tox}_left.webp"
							alt=""
						/>{/if}
					{#if row.image}<img class="tox-icon" src="/art/tox/{row.id}.webp" alt="" />{/if}
					<span class="text tox-name" style:--n={row.name.length}>{row.name}</span>
					{#if row.tox}<img
							src="/art/ui/card_ability_passive_line_{row.tox}_right.webp"
							alt=""
						/>{/if}
				</span>
			{/if}
		</div>
	{/each}
	<div class="hp"></div>
	<span class="text hp-amount"><span class="sr-only">{m.hp()}</span> {card.hp}/{card.hp}</span>
	<img class="gold" src="/art/ui/gold_icon.webp" alt="" />
	<span class="gold-amount"><span class="sr-only">{m.income()}</span> {card.goldIncome}</span>
</div>

<style>
	.card {
		position: relative;
		aspect-ratio: 1432 / 2048;
		container-type: inline-size;
		font-family: Raleway, sans-serif;
		font-weight: 800;
		font-variant-numeric: lining-nums;
		line-height: 1;
	}
	.card > *,
	.masked > *,
	.slot > * {
		position: absolute;
	}
	.rim {
		inset: -1.201% -1.753%;
		background: #000;
		mask: url(/art/ui/card_mask.webp) 0 0 / 100% 100%;
	}
	.masked {
		inset: 0;
		overflow: hidden;
		background: color-mix(in oklab, var(--primary) 40%, #000);
		mask: url(/art/ui/card_mask.webp) 0 0 / 100% 100%;
	}
	.art {
		top: -0.83%;
		left: 0;
		width: 100%;
		height: 101.665%;
		max-width: none;
	}
	.header {
		inset: 0 0 90%;
		background: linear-gradient(#000, transparent);
	}
	.fill {
		inset: 0;
		width: 100%;
		height: 100%;
	}
	.text {
		display: flex;
		align-items: center;
		justify-content: center;
		color: #fff;
		text-align: center;
		text-wrap: balance;
		-webkit-text-stroke: 0.14em #000;
		paint-order: stroke fill;
		text-shadow: 0 0.03em 0.12em rgb(0 0 0 / 0.7);
	}
	.name {
		top: 2.5%;
		left: 20%;
		width: 65%;
		height: 4%;
		font-size: min(4.888cqw, 65cqw / (var(--n) * 0.6));
	}
	.badge {
		top: -5%;
		left: -9.511%;
		width: 25.028%;
		height: 17.5%;
	}
	.gem {
		top: 95.254%;
		left: 35%;
		width: 30%;
		height: 10.488%;
	}
	.slot {
		left: 2.5%;
		width: 95%;
		height: 10.64%;
	}
	.ability-bg {
		inset: 0;
		background: rgb(0 0 0 / 0.25);
		mask: url(/art/ui/card_ability_active.webp) 0 0 / 100% 100%;
	}
	.ability-element {
		top: 14.089%;
		left: 4.998%;
		width: 11.497%;
		height: 71.776%;
	}
	.aura {
		inset: 0;
		position: absolute;
		background: radial-gradient(closest-side, var(--primary), transparent);
	}
	.ability-element img {
		position: absolute;
		inset: 4.987%;
		width: 90.026%;
		height: 90.026%;
	}
	.ability-name {
		top: 25.01%;
		left: 17.502%;
		width: 62.496%;
		height: 49.977%;
		font-size: min(5.587cqw, 59.37cqw / (var(--n) * 0.6));
	}
	.cooldown {
		top: 10.005%;
		left: 80.998%;
		width: 9.997%;
		height: 79.99%;
		background: var(--secondary);
		mask: url(/art/ui/card_cooldown.webp) right center / contain no-repeat;
	}
	.cooldown-amount {
		top: 25.975%;
		left: 80.998%;
		width: 9.997%;
		height: 48.004%;
		font-size: 6.774cqw;
	}
	.tox-aura {
		top: 25.01%;
		left: 0;
		width: 100%;
		height: 49.977%;
		opacity: 0.3;
		background: radial-gradient(closest-side, var(--rarity), transparent);
	}
	.tox {
		top: 25.01%;
		left: 0;
		width: 100%;
		height: 49.977%;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 1.5cqw;
	}
	.tox img {
		flex: 0 1 22.82cqw;
		min-width: 0;
		height: 11.41cqw;
		object-fit: contain;
	}
	.tox .tox-icon {
		flex: none;
		width: 9cqw;
		height: 9cqw;
	}
	.tox-name {
		flex: none;
		max-width: 59.37cqw;
		font-size: min(5.587cqw, 59.37cqw / (var(--n) * 0.6));
	}
	.hp {
		top: 91.011%;
		left: 4.197%;
		width: 91.606%;
		height: 5.508%;
		background: #80cc4d;
		mask: url(/art/ui/card_hp_bar_mask.webp) 0 0 / 100% 100%;
	}
	.hp-amount {
		top: 91.172%;
		left: 3.499%;
		width: 93.003%;
		height: 5.181%;
		font-size: 5.587cqw;
	}
	.gold {
		top: -2.002%;
		left: 90%;
		width: 15.999%;
		height: 12.002%;
		object-fit: contain;
	}
	.gold-amount {
		top: 1.484%;
		left: 94.399%;
		width: 7.2%;
		height: 5.034%;
		display: flex;
		align-items: center;
		justify-content: center;
		color: #984300;
		font-size: 6.285cqw;
	}
	[data-element='fire'] {
		--primary: #bf381d;
		--secondary: #fc873e;
	}
	[data-element='light'] {
		--primary: #fef182;
		--secondary: #fff;
	}
	[data-element='earth'] {
		--primary: #7b5f47;
		--secondary: #bb9981;
	}
	[data-element='nature'] {
		--primary: #617a21;
		--secondary: #96b03c;
	}
	[data-element='shadow'] {
		--primary: #532679;
		--secondary: #b620ff;
	}
	[data-element='water'] {
		--primary: #4ad0fd;
		--secondary: #aff7ff;
	}
	[data-element='divine'] {
		--primary: #000;
		--secondary: #aaa;
	}
	[data-rarity='common'] {
		--rarity: #333;
	}
	[data-rarity='rare'] {
		--rarity: #0050ff;
	}
	[data-rarity='epic'] {
		--rarity: #8939fd;
	}
	[data-rarity='legendary'] {
		--rarity: #fe951b;
	}
</style>
