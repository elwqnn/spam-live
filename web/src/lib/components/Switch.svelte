<script lang="ts">
	import type { HTMLButtonAttributes } from 'svelte/elements';

	let {
		checked,
		class: className,
		...rest
	}: { checked: boolean } & Omit<HTMLButtonAttributes, 'type' | 'role'> = $props();
</script>

<button
	type="button"
	role="switch"
	aria-checked={checked}
	class={[
		'group relative h-5.5 w-9.5 flex-none cursor-pointer rounded-sm border border-foreground/25 transition-colors aria-checked:border-primary aria-checked:bg-primary',
		className
	]}
	{...rest}
>
	<span
		class="absolute top-0.5 left-0.5 size-4 rounded-full bg-foreground transition-[translate,background-color] duration-150 ease-out group-aria-checked:translate-x-4 group-aria-checked:bg-primary-foreground"
	></span>
</button>
