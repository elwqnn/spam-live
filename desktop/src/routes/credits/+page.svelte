<script lang="ts">
	import CreditsView from '@spam-live/ui/views/CreditsView.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<CreditsView {data} />
