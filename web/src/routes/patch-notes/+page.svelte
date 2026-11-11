<script lang="ts">
	import PatchNotesView from '@spam-live/ui/views/PatchNotesView.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<PatchNotesView {data} />
