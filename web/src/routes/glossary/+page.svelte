<script lang="ts">
	import GlossaryView from '@spam-live/ui/views/GlossaryView.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<GlossaryView {data} />
