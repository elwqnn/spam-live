<script lang="ts">
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { formatDescription } from '#lib/description.js';
	import { m } from '#lib/paraglide/messages.js';
	import CardBadge from './CardBadge.svelte';

	let { text, entry }: { text?: string | null; entry: { data: object } } = $props();

	const segments = $derived(text ? formatDescription(text, entry) : []);
</script>

<p class="text-sm leading-6">
	{#each segments as s, i (i)}{#if s.type === 'text'}{s.text}{:else if s.type === 'br'}<br
			/>{:else if s.type === 'element'}<CardBadge
				type="element"
				value={s.element}
				label={s.label}
			/>{:else if s.type === 'keyword'}<Badge variant="secondary">{s.text}</Badge
			>{:else if s.type === 'value'}<span
				class={s.dynamic
					? 'cursor-help font-semibold underline decoration-dotted underline-offset-4'
					: 'font-semibold'}
				title={s.dynamic ? m.base_value() : undefined}>{s.text}</span
			>{:else}<span class="text-muted-foreground">[{s.name}]</span>{/if}{/each}
</p>
