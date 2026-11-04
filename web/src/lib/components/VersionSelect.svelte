<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { queryHref } from '#lib';
	import * as NativeSelect from '#lib/components/ui/native-select/index.js';
	import { m } from '#lib/paraglide/messages.js';

	let { versions }: { versions: string[] } = $props();

	const value = $derived(page.url.searchParams.get('version') ?? versions[0] ?? 'all');
</script>

<label class="flex items-center gap-2 text-sm">
	<span class="eyebrow text-muted-foreground">{m.version()}</span>
	<NativeSelect.Root
		{value}
		onchange={(e) =>
			goto(
				queryHref(page.url, {
					version: e.currentTarget.value === versions[0] ? null : e.currentTarget.value,
					...(e.currentTarget.value === 'all' ? { trend: null } : {})
				}),
				{ reset: false }
			)}
	>
		{#each versions as v (v)}
			<NativeSelect.Option value={v}>{v}</NativeSelect.Option>
		{/each}
		<NativeSelect.Option value="all">{m.all_versions()}</NativeSelect.Option>
	</NativeSelect.Root>
</label>
