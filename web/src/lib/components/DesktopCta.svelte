<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Download04Icon } from '@hugeicons/core-free-icons';
	import { href, type Schemas } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import { Button } from './ui/button/index.js';

	let {
		version,
		cards,
		catalog
	}: {
		version?: string | null;
		cards: string[];
		catalog: Map<string, Schemas['CardBase']>;
	} = $props();

	const shape = '[mask:url(/art/ui/card_mask.webp)_0_0/100%_100%]';
</script>

<section class="grid items-center gap-10 bg-straw row text-[#171715] lg:grid-cols-2 lg:gap-8">
	<div class="flex flex-col items-start gap-5">
		<p class="eyebrow">{m.download_title()}</p>
		<h2
			class="font-heading text-4xl/none font-normal tracking-[-0.025em] text-balance italic sm:text-5xl/none xl:text-6xl/none"
		>
			{m.home_app_title()}
		</h2>
		<p class="max-w-md text-lg text-balance">{m.home_app_text()}</p>
		{#if version}
			<p class="text-sm tabular-nums">{m.download_version({ version })}</p>
		{/if}
		<Button
			size="lg"
			href={href('/download')}
			class="border-[#171715] bg-[#171715] text-straw hover:bg-[#2a2925] focus-visible:border-[#171715] focus-visible:ring-[#171715]/50"
			><HugeiconsIcon icon={Download04Icon} />{m.nav_download()}</Button
		>
	</div>
	<div class="fan relative w-full" aria-hidden="true">
		{#each cards as id, i (id)}
			<span
				class="leaf absolute el-{catalog.get(id)?.element}"
				style:--a="{(i - (cards.length - 1) / 2) * 7}deg"
			>
				<span class={['absolute -inset-0.5 bg-(--el)', shape]}></span>
				<span class={['absolute inset-0 bg-[color-mix(in_oklab,var(--el)_40%,#000)]', shape]}>
					<img
						src="/art/thumb/{id}.webp"
						alt=""
						loading="lazy"
						decoding="async"
						class="size-full object-cover"
					/>
				</span>
			</span>
		{/each}
	</div>
</section>

<style>
	.fan {
		--w: min(11rem, 36vw);
		height: calc(var(--w) * 1.65);
		view-timeline: --fan block;
	}

	@media (width >= 64rem) {
		.fan {
			--w: min(13rem, 18vw);
		}
	}

	.leaf {
		top: calc(var(--w) * 0.05);
		left: calc(50% - var(--w) / 2);
		width: var(--w);
		aspect-ratio: 1432 / 2048;
		transform-origin: 50% 200%;
		rotate: var(--a);
	}

	@supports (animation-timeline: view()) {
		@media (prefers-reduced-motion: no-preference) {
			.leaf {
				animation: open linear both;
				animation-timeline: --fan;
				animation-range: entry 10% entry 100%;
			}
		}
	}

	@keyframes open {
		from {
			rotate: 0deg;
		}
	}
</style>
