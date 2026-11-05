<script lang="ts">
	let {
		id,
		image,
		lazy = true,
		class: className
	}: { id: string; image: boolean; lazy?: boolean; class?: string } = $props();
</script>

<span class={['block shrink-0', className]}>
	{#if image}
		<img
			src="/art/tox/{id}.webp"
			alt=""
			aria-hidden="true"
			loading={lazy ? 'lazy' : undefined}
			decoding="async"
			class="size-full"
		/>
	{/if}
</span>
