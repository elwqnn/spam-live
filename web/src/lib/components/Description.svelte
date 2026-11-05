<script lang="ts">
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { formatDescription, matchKeyword, type Segment } from '#lib/description.js';
	import { m } from '#lib/paraglide/messages.js';
	import CardBadge from './CardBadge.svelte';
	import KeywordHint from './KeywordHint.svelte';
	import Tooltip from './Tooltip.svelte';

	type Keyword = { id: string; name: string; description?: string | null };

	let {
		text,
		entry,
		keywords = []
	}: { text?: string | null; entry: { data: object }; keywords?: readonly Keyword[] } = $props();

	const segments = $derived(text ? formatDescription(text, entry) : []);
</script>

{#snippet render(list: Segment[], linked: boolean)}
	{#each list as s, i (i)}{#if s.type === 'text'}{s.text}{:else if s.type === 'br'}<br
			/>{:else if s.type === 'element'}<CardBadge
				type="element"
				value={s.element}
				label={s.label}
			/>{:else if s.type === 'keyword'}{@const keyword = linked
				? matchKeyword(s.text, keywords)
				: undefined}{#if keyword}<KeywordHint {keyword} label={s.text}
					>{@render render(
						keyword.description ? formatDescription(keyword.description, { data: {} }) : [],
						false
					)}</KeywordHint
				>{:else}<Badge variant="secondary">{s.text}</Badge
				>{/if}{:else if s.type === 'value'}{#if s.dynamic}<Tooltip content={m.base_value()}
					>{#snippet children(tip)}<span
							class="cursor-help font-semibold underline decoration-dotted underline-offset-4"
							{...tip}>{s.text}</span
						>{/snippet}</Tooltip
				>{:else}<span class="font-semibold">{s.text}</span>{/if}{:else}<span
				class="text-muted-foreground">[{s.name}]</span
			>{/if}{/each}
{/snippet}

<p class="text-sm leading-6">{@render render(segments, true)}</p>
