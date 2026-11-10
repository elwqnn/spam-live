<script lang="ts" module>
	import type { Tier } from '#lib/tier.js';

	const TONES: Record<Tier, string> = {
		S: 'bg-terra text-background',
		A: 'bg-straw text-background',
		B: 'bg-moss text-background',
		C: 'bg-slate text-background',
		D: 'bg-plum text-foreground'
	};
</script>

<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import { cn } from '#lib/utils.js';

	let {
		tier,
		large = false,
		stretch = false,
		class: className
	}: { tier: Tier; large?: boolean; stretch?: boolean; class?: string } = $props();
</script>

<span
	class={cn(
		'inline-grid shrink-0 place-items-center rounded-xs font-sans leading-none font-semibold not-italic',
		stretch
			? 'w-12 content-start self-stretch pt-5 text-2xl sm:w-16'
			: large
				? 'size-10 text-lg'
				: 'size-5.5 text-xs',
		TONES[tier],
		className
	)}
	><span
		aria-hidden="true"
		class={[stretch && 'sticky top-5 sm:top-[calc(var(--head,7rem)+1.25rem)]']}>{tier}</span
	><span class="sr-only">{m.tier_label({ tier })}</span></span
>
