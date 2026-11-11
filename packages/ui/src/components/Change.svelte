<script lang="ts">
	import { pageUrl } from '#ui/modal.js';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowDown01Icon, ArrowUp01Icon } from '@hugeicons/core-free-icons';
	import { formatChange } from '#ui';
	import { cn } from '#ui/utils.js';
	import { m } from '$paraglide/messages.js';
	import { getLocale } from '$paraglide/runtime.js';

	let {
		value,
		label,
		color = false,
		class: className
	}: { value?: number | null; label?: string; color?: boolean; class?: string } = $props();

	const change = $derived(formatChange(value, getLocale()));
	const against = $derived(
		pageUrl().searchParams.get('trend') === 'version' &&
			pageUrl().searchParams.get('version') !== 'all'
			? m.against_version()
			: m.against_week()
	);
</script>

{#if change}
	<span
		class={cn(
			'whitespace-nowrap tabular-nums',
			color && change.sign > 0
				? 'text-up'
				: color && change.sign < 0
					? 'text-down'
					: 'text-muted-foreground',
			className
		)}
	>
		<span aria-hidden="true"
			>{#if change.sign > 0}<HugeiconsIcon
					icon={ArrowUp01Icon}
					aria-hidden="true"
					class="mr-[.25em] inline size-[1em] align-[-0.125em]"
				/>{:else if change.sign < 0}<HugeiconsIcon
					icon={ArrowDown01Icon}
					aria-hidden="true"
					class="mr-[.25em] inline size-[1em] align-[-0.125em]"
				/>{/if}{change.text}</span
		>
		<span class="sr-only"
			>{label}
			{change.sign > 0
				? m.change_up({ value: change.value, against })
				: change.sign < 0
					? m.change_down({ value: change.value, against })
					: m.change_none({ against })}</span
		>
	</span>
{/if}
