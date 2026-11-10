<script lang="ts">
	let { diagonal = false, class: className }: { diagonal?: boolean; class?: string } = $props();
</script>

<svg
	data-icon="inline-end"
	class={className}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="2"
	stroke-linecap="round"
	stroke-linejoin="round"
	aria-hidden="true"><path d={diagonal ? 'M7 17L17 7M9 7h8v8' : 'M5 12h14M13 6l6 6-6 6'} /></svg
>
