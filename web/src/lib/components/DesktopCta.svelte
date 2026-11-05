<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Download04Icon } from '@hugeicons/core-free-icons';
	import { href } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
	import ShaderBackground from './ShaderBackground.svelte';
	import { Button } from './ui/button/index.js';

	let { version }: { version?: string | null } = $props();
</script>

<section class="track">
	<div class="stage relative isolate flex items-center justify-center text-[#fbf8f0]">
		<div class="frame absolute inset-0 overflow-hidden">
			<div class="media absolute inset-0"><ShaderBackground /></div>
		</div>
		<div
			class="relative flex w-full flex-col items-center gap-5 px-(--gutter) py-20 text-center sm:py-24"
		>
			<h2
				class="title font-heading text-3xl/tight font-normal tracking-[-0.025em] text-balance italic sm:text-4xl/tight xl:text-5xl/tight"
			>
				{m.home_app_title()}
			</h2>
			<div class="rest flex flex-col items-center gap-5">
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
	</div>
</section>

<style>
	@supports (animation-timeline: view()) {
		@media (prefers-reduced-motion: no-preference) {
			.track {
				--stage: 100svh;
				--top: 0px;
				--x: 6%;
				--y: 20%;
				height: calc(var(--stage) * 2.4);
				view-timeline: --cta block;
				view-timeline-inset: var(--top) 0px;
			}
			@media (width >= 40rem) {
				.track {
					--stage: calc(100svh - var(--head, 7rem));
					--top: var(--head, 7rem);
				}
			}
			@media (width >= 64rem) {
				.track {
					--x: 29%;
					--y: 21%;
				}
			}
			.stage {
				position: sticky;
				top: var(--top);
				z-index: 10;
				width: 100vw;
				height: var(--stage);
				margin-left: calc(50% - 50vw);
			}
			.frame,
			.media,
			.title,
			.rest {
				animation-timing-function: linear;
				animation-fill-mode: both;
				animation-timeline: --cta;
				animation-range: contain 0% contain 80%;
			}
			.frame {
				animation-name: expand;
			}
			.media {
				animation-name: zoom;
			}
			.title {
				max-width: calc(100% - 2 * var(--x));
				animation-name: lift;
			}
			.rest {
				animation-name: reveal;
				animation-range: contain 55% contain 80%;
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

	@keyframes lift {
		from {
			translate: 0 5rem;
		}
		to {
			translate: 0 0;
		}
	}

	@keyframes reveal {
		from {
			opacity: 0;
			visibility: hidden;
			translate: 0 1rem;
		}
		60% {
			visibility: hidden;
		}
		to {
			opacity: 1;
			visibility: visible;
			translate: 0 0;
		}
	}
</style>
