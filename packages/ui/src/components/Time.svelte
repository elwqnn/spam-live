<script lang="ts">
	import { formatDate } from '#ui';
	import { getLocale } from '$paraglide/runtime.js';
	import Tooltip from './Tooltip.svelte';

	let { iso, class: className }: { iso: string; class?: string } = $props();

	const full = $derived(
		new Intl.DateTimeFormat(getLocale(), {
			dateStyle: 'long',
			timeStyle: 'long',
			timeZone: 'UTC'
		}).format(new Date(iso))
	);
</script>

<Tooltip content={full}
	>{#snippet children(tip)}<time datetime={iso} class={className} {...tip}>{formatDate(iso)}</time
		>{/snippet}</Tooltip
>
