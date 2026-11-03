<script lang="ts" module>
	import { type VariantProps, tv } from 'tailwind-variants';

	export const badgeVariants = tv({
		base: 'inline-flex w-fit shrink-0 items-center gap-1 overflow-hidden rounded-sm border border-transparent px-1.5 py-0.5 font-mono text-[10px] leading-normal font-medium tracking-[.06em] whitespace-nowrap uppercase',
		variants: {
			variant: {
				default: 'bg-primary text-primary-foreground',
				el: 'bg-(--el) text-background',
				secondary: 'bg-secondary text-secondary-foreground',
				destructive: 'bg-destructive/20 text-destructive',
				outline: 'border-white/20 text-muted-foreground'
			}
		},
		defaultVariants: {
			variant: 'default'
		}
	});

	export type BadgeVariant = VariantProps<typeof badgeVariants>['variant'];
</script>

<script lang="ts">
	import { cn, type WithElementRef } from '#lib/utils.js';
	import type { HTMLAnchorAttributes } from 'svelte/elements';

	let {
		ref = $bindable(null),
		href,
		class: className,
		variant = 'default',
		children,
		...restProps
	}: WithElementRef<HTMLAnchorAttributes> & {
		variant?: BadgeVariant;
	} = $props();
</script>

<svelte:element
	this={href ? 'a' : 'span'}
	bind:this={ref}
	data-slot="badge"
	{href}
	class={cn(badgeVariants({ variant }), className)}
	{...restProps}
>
	{@render children?.()}
</svelte:element>
