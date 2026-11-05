<script lang="ts" module>
	import type { Filter } from '#lib/filter.js';
	import { m } from '#lib/paraglide/messages.js';

	export const versionFilter = (versions: string[]): Filter => ({
		id: 'version',
		label: m.version(),
		default: versions[0] ?? 'all',
		options: [
			...versions.map((v) => ({ value: v, label: v })),
			{ value: 'all', label: m.all_versions(), reset: { trend: null } }
		]
	});
</script>

<script lang="ts">
	import FilterMenu from './FilterMenu.svelte';

	let { versions }: { versions: string[] } = $props();
</script>

<FilterMenu filter={versionFilter(versions)} />
