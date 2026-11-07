<script lang="ts" module>
	import type { Tier } from '#lib/tier.js';

	export const TONES: Record<Tier, string> = {
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

	let { tier, class: className }: { tier: Tier; class?: string } = $props();
</script>

<span
	class={cn(
		'inline-grid h-5.5 min-w-5.5 shrink-0 place-items-center rounded-xs px-1 text-xs leading-none font-semibold',
		TONES[tier],
		className
	)}
	><span aria-hidden="true">{tier}</span><span class="sr-only">{m.tier_label({ tier })}</span></span
>
