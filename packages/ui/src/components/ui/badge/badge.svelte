<script lang="ts" module>
	import { type VariantProps, tv } from 'tailwind-variants';

	export const badgeVariants = tv({
		base: 'group/badge inline-flex h-5 w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded-xs border border-transparent px-1.5 text-[11px] font-medium tracking-[.06em] whitespace-nowrap uppercase transition-colors has-data-[icon=inline-end]:pr-1 has-data-[icon=inline-start]:pl-1 focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring aria-invalid:border-destructive aria-invalid:ring-destructive/40 [&>svg]:pointer-events-none [&>svg]:size-3!',
		variants: {
			variant: {
				default: 'bg-primary text-primary-foreground [a]:hover:bg-primary/90',
				secondary: 'bg-secondary text-secondary-foreground [a]:hover:bg-secondary/80',
				destructive: 'bg-destructive/20 text-destructive focus-visible:ring-destructive/40',
				outline: 'border-foreground/20 text-foreground [a]:hover:bg-muted',
				ghost: 'hover:bg-muted hover:text-foreground',
				link: 'text-primary underline-offset-4 hover:underline'
			}
		},
		defaultVariants: {
			variant: 'default'
		}
	});

	export type BadgeVariant = VariantProps<typeof badgeVariants>['variant'];
</script>

<script lang="ts">
	import { cn, type WithElementRef } from '#ui/utils.js';
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
