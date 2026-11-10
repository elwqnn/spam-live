<script lang="ts">
	import { page } from '$app/state';
	import { href } from '#lib';
	import TitleRow from '@spam-live/ui/components/TitleRow.svelte';
	import { m } from '#lib/paraglide/messages.js';
</script>

<svelte:head>
	<title>{page.status} · {m.site_name()}</title>
</svelte:head>

<TitleRow class="grid justify-items-center gap-3 px-(--gutter) py-16 text-center">
	<p class="text-5xl tabular-nums">{page.status}</p>
	<h1 class="font-heading text-3xl font-normal tracking-tight">
		{page.status === 404 ? m.error_not_found() : m.error_generic()}
	</h1>
	<a class="eyebrow text-muted-foreground transition-colors hover:text-foreground" href={href('/')}
		>{m.back_home()}</a
	>
</TitleRow>
