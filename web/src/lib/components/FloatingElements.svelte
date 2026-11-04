<script lang="ts">
	import { ELEMENTS } from './CardBadge.svelte';

	type Icon = [x: number, y: number, depth: number, duration: number, delay: number];

	const KEYS = Object.keys(ELEMENTS);
	const SIDES: [string, number, Icon[]][] = [
		[
			'left-0 right-[calc(50%+37.5rem+var(--side))]',
			0,
			[
				[38, 9, 0.25, 9, 2],
				[51, 23, 0.85, 13, 7],
				[32, 39, 0.45, 7, 4],
				[66, 54, 0.1, 11, 9],
				[50, 68, 1, 14, 3],
				[70, 82, 0.35, 8, 6],
				[38, 94, 0.6, 12, 10]
			]
		],
		[
			'right-0 left-[calc(50%+37.5rem+var(--side))]',
			3,
			[
				[58, 13, 0.7, 12, 5],
				[32, 29, 0.2, 8, 1],
				[66, 42, 0.4, 10, 8],
				[50, 57, 0.95, 14, 11],
				[70, 71, 0.15, 6, 3],
				[44, 84, 0.5, 9, 7],
				[64, 96, 0.3, 11, 4]
			]
		]
	];
</script>

<div
	class="pointer-events-none fixed inset-0 hidden opacity-30 min-[1440px]:block dark:opacity-35"
	aria-hidden="true"
>
	{#each SIDES as [place, offset, icons] (place)}
		<div class={['@container absolute inset-y-0 overflow-hidden', place]}>
			{#each icons as [left, top, depth, duration, delay], i (i)}
				<div class="orb absolute" style:left="{left}%" style:top="{top}%" style:--d={depth}>
					<img
						src="/art/ui/element_{KEYS[(i + offset) % KEYS.length]}.webp"
						alt=""
						loading="lazy"
						style:animation-duration="{duration}s"
						style:animation-delay="-{delay}s"
					/>
				</div>
			{/each}
		</div>
	{/each}
</div>

<style>
	.orb {
		width: clamp(32px, 12cqw + var(--d) * 28cqw, 160px);
		aspect-ratio: 1;
		transform: translate(-50%, -50%);
	}
	img {
		display: block;
		width: 100%;
		height: 100%;
		filter: blur(calc(2px + var(--d) * 8px));
	}
	@media (prefers-reduced-motion: no-preference) {
		img {
			animation-name: float;
			animation-timing-function: ease-in-out;
			animation-iteration-count: infinite;
			animation-direction: alternate;
		}
	}
	@keyframes float {
		from {
			transform: translateY(-8%) rotate(-6deg);
		}
		to {
			transform: translateY(8%) rotate(6deg);
		}
	}
</style>
