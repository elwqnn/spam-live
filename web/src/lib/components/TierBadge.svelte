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
		class: className
	}: { tier: Tier; large?: boolean; class?: string } = $props();
</script>

<span
	class={cn(
		'inline-grid shrink-0 place-items-center rounded-xs font-sans leading-none font-semibold not-italic',
		large ? 'size-10 text-lg' : 'size-5.5 text-xs',
		TONES[tier],
		className
	)}
	><span aria-hidden="true">{tier}</span><span class="sr-only">{m.tier_label({ tier })}</span></span
>
