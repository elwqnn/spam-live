<script lang="ts">
	import { writes } from '#lib/host.js';
	import MeView from '@spam-live/ui/views/MeView.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<MeView {data} {writes} />
