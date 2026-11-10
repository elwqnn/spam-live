<script lang="ts">
	import { ELEMENTS } from '#ui/components/CardBadge.svelte';
	import { art } from '$host';
</script>

<ul class="flex flex-wrap gap-x-4 gap-y-1 px-(--gutter) pt-2">
	{#each Object.entries(ELEMENTS) as [key, name] (key)}
		<li class="flex items-center gap-1.5 text-xs text-muted-foreground">
			<img
				src={art(`ui/element_${key}.webp`)}
				alt=""
				aria-hidden="true"
				decoding="async"
				class="size-4"
			/>
			{name()}
		</li>
	{/each}
</ul>
