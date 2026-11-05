<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { UnfoldMoreIcon } from '@hugeicons/core-free-icons';
	import { cn, type WithElementRef } from '#lib/utils.js';
	import type { HTMLSelectAttributes } from 'svelte/elements';

	type NativeSelectProps = Omit<WithElementRef<HTMLSelectAttributes, HTMLSelectElement>, 'size'> & {
		size?: 'sm' | 'default';
	};

	let {
		ref = $bindable(null),
		value = $bindable(),
		class: className,
		size = 'default',
		children,
		...restProps
	}: NativeSelectProps = $props();
</script>

<div
	class={cn('group/native-select relative w-fit has-[select:disabled]:opacity-50', className)}
	data-slot="native-select-wrapper"
	data-size={size}
>
	<select
		bind:value
		bind:this={ref}
		data-slot="native-select"
		data-size={size}
		class="h-9 w-full min-w-0 appearance-none rounded-sm border border-input bg-card py-1 pr-8 pl-3 text-sm transition-colors outline-none select-none selection:bg-primary selection:text-primary-foreground placeholder:text-muted-foreground hover:border-foreground/25 focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:cursor-not-allowed aria-invalid:border-destructive/50 aria-invalid:ring-1 aria-invalid:ring-destructive/40 data-[size=sm]:h-8"
		{...restProps}
	>
		{@render children?.()}
	</select>
	<HugeiconsIcon
		icon={UnfoldMoreIcon}
		strokeWidth={2}
		class="pointer-events-none absolute top-1/2 right-3.5 size-4 -translate-y-1/2 text-muted-foreground select-none"
		aria-hidden
		data-slot="native-select-icon"
	/>
</div>
