<script lang="ts">
	import { page } from '$app/state';
	import { formatChange } from '#lib';
	import { cn } from '#lib/utils.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';

	let {
		value,
		label,
		color = false,
		class: className
	}: { value?: number | null; label?: string; color?: boolean; class?: string } = $props();

	const change = $derived(formatChange(value, getLocale()));
	const against = $derived(
		page.url.searchParams.get('trend') === 'version' &&
			page.url.searchParams.get('version') !== 'all'
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
		<span aria-hidden="true">{change.text}</span>
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
