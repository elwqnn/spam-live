<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Download04Icon } from '@hugeicons/core-free-icons';
	import { href } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import ShaderBackground from './ShaderBackground.svelte';
	import { Button } from './ui/button/index.js';

	let { version }: { version?: string | null } = $props();

	const tagline = m.home_app_tagline();
</script>

<section class="track" data-cta>
	<div class="snap" aria-hidden="true"></div>
	<div class="stage relative isolate flex items-center justify-center text-[#fbf8f0]">
		<div class="frame absolute inset-0 overflow-hidden">
			<div class="media absolute inset-0"><ShaderBackground /></div>
		</div>
		<p
			aria-hidden="true"
			class="tagline absolute font-heading leading-none tracking-[-0.025em] whitespace-nowrap italic"
			style:--n={tagline.length}
		>
			{tagline}
		</p>
		<div
			class="rest relative flex w-full flex-col items-center gap-5 px-(--gutter) py-20 text-center sm:py-24"
		>
			<h2
				class="font-heading text-3xl/tight font-normal tracking-[-0.025em] text-balance italic sm:text-4xl/tight xl:text-5xl/tight"
			>
				{m.home_app_title()}
			</h2>
			<p class="max-w-xl text-lg text-balance">{m.home_app_text()}</p>
			{#if version}
				<p class="text-sm tabular-nums">{m.download_version({ version })}</p>
			{/if}
			<Button
				size="lg"
				href={href('/download')}
				class="border-[#f2ead3] bg-[#f2ead3] text-[#171715] hover:bg-[#fbf4df]"
				><HugeiconsIcon icon={Download04Icon} />{m.nav_download()}</Button
			>
		</div>
	</div>
</section>

<style>
	.tagline {
		display: none;
		font-size: calc(min(100vw - 2 * var(--gutter), 1.2 * var(--bw)) / (var(--n) * 0.39));
	}

	@supports (animation-timeline: view()) {
		@media (prefers-reduced-motion: no-preference) {
			:global(html:has([data-cta])) {
				scroll-snap-type: y proximity;
			}
			.track {
				--stage: 100lvh;
				--x: 14%;
				--bw: 72vw;
				--y: 42%;
				position: relative;
				height: calc(var(--stage) * 2.5);
				view-timeline: --cta block;
			}
			@media (width >= 64rem) {
				.track {
					--x: 22%;
					--bw: 56vw;
					--y: 39%;
				}
			}
			.snap {
				position: absolute;
				top: var(--stage);
				right: 0;
				left: 0;
				height: 1px;
				scroll-snap-align: start;
				pointer-events: none;
			}
			@media (width >= 40rem) {
				.snap {
					top: calc(var(--stage) + 7rem);
				}
			}
			.stage {
				position: sticky;
				top: 0;
				z-index: 10;
				width: 100vw;
				height: var(--stage);
				margin-left: calc(50% - 50vw);
			}
			.tagline {
				display: block;
			}
			.frame,
			.media,
			.tagline,
			.rest {
				animation-timing-function: linear;
				animation-fill-mode: both;
				animation-timeline: --cta;
				animation-range: contain 0% contain 66.67%;
			}
			.frame {
				animation-name: expand;
			}
			.media {
				animation-name: zoom;
			}
			.tagline {
				animation-name: fade;
				animation-range: contain 26.67% contain 58.67%;
			}
			.rest {
				animation-range: contain 45.33% contain 66.67%;
			}
			.rest:not(:focus-within) {
				animation-name: reveal;
			}
			.stage:has(.rest:focus-within) .tagline {
				animation: none;
				opacity: 0;
			}
			@supports (timeline-scope: --cta) {
				:global(html:has([data-cta])) {
					timeline-scope: --cta;
				}
				@media (width >= 40rem) {
					:global(html:has([data-cta]) header.head:not(:focus-within)) {
						animation-name: hide;
						animation-timing-function: linear;
						animation-fill-mode: both;
						animation-timeline: --cta;
						animation-range: contain 56.67% contain 66.67%;
					}
				}
			}
		}
	}

	@keyframes expand {
		from {
			clip-path: inset(var(--y) var(--x));
		}
		to {
			clip-path: inset(0);
		}
	}

	@keyframes zoom {
		from {
			scale: 1.35;
		}
		to {
			scale: 1;
		}
	}

	@keyframes fade {
		from {
			opacity: 1;
			translate: 0 0;
			scale: 1;
		}
		to {
			opacity: 0;
			translate: 0 -2rem;
			scale: 1.06;
		}
	}

	@keyframes reveal {
		from {
			opacity: 0;
			pointer-events: none;
			translate: 0 1.125rem;
		}
		60% {
			pointer-events: none;
		}
		to {
			opacity: 1;
			pointer-events: auto;
			translate: 0 0;
		}
	}

	@keyframes hide {
		to {
			translate: 0 calc(-100% - 5px);
		}
	}
</style>
