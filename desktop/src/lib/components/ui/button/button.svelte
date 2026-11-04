<script lang="ts" module>
	import { type VariantProps, tv } from 'tailwind-variants';
	import { cn, type WithElementRef } from '#lib/utils.js';
	import type { HTMLAnchorAttributes, HTMLButtonAttributes } from 'svelte/elements';

	export const buttonVariants = tv({
		base: "group/button inline-flex shrink-0 cursor-pointer items-center justify-center gap-2 rounded-sm border text-[11px] font-medium tracking-[.06em] whitespace-nowrap uppercase transition-colors select-none focus-visible:outline-2 focus-visible:outline-offset-2 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-3",
		variants: {
			variant: {
				default:
					'relative isolate overflow-hidden border-primary bg-primary text-primary-foreground hover:bg-primary/90',
				outline: 'border-white/18 text-foreground hover:border-white/40',
				secondary: 'border-white/10 bg-secondary text-secondary-foreground hover:bg-row-hover',
				ghost: 'border-transparent text-foreground hover:border-border hover:bg-row-hover',
				destructive:
					'border-destructive/30 bg-destructive/20 text-destructive hover:bg-destructive/30',
				link: 'border-transparent text-primary underline-offset-4 hover:underline'
			},
			size: {
				default: 'h-[30px] px-3',
				sm: 'h-7 px-2.5',
				lg: 'h-9 px-4',
				icon: 'size-[30px]'
			}
		},
		defaultVariants: {
			variant: 'default',
			size: 'default'
		}
	});

	export type ButtonVariant = VariantProps<typeof buttonVariants>['variant'];
	export type ButtonSize = VariantProps<typeof buttonVariants>['size'];

	export type ButtonProps = WithElementRef<HTMLButtonAttributes> &
		WithElementRef<HTMLAnchorAttributes> & {
			variant?: ButtonVariant;
			size?: ButtonSize;
		};
</script>

<script lang="ts">
	import LiveLines from '#lib/components/LiveLines.svelte';

	let {
		class: className,
		variant = 'default',
		size = 'default',
		ref = $bindable(null),
		href = undefined,
		type = 'button',
		disabled,
		children,
		...restProps
	}: ButtonProps = $props();
</script>

{#if href}
	<a
		bind:this={ref}
		data-slot="button"
		class={cn(buttonVariants({ variant, size }), className)}
		href={disabled ? undefined : href}
		aria-disabled={disabled}
		role={disabled ? 'link' : undefined}
		tabindex={disabled ? -1 : undefined}
		{...restProps}
	>
		{#if variant === 'default'}<LiveLines />{/if}
		{@render children?.()}
	</a>
{:else}
	<button
		bind:this={ref}
		data-slot="button"
		class={cn(buttonVariants({ variant, size }), className)}
		{type}
		{disabled}
		{...restProps}
	>
		{#if variant === 'default'}<LiveLines />{/if}
		{@render children?.()}
	</button>
{/if}
