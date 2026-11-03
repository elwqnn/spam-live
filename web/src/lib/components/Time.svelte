<script lang="ts">
	import { formatDate } from '#lib';

	let { iso, class: className }: { iso: string; class?: string } = $props();
</script>

<time datetime={iso} title={iso} class={className}>{formatDate(iso)}</time>
