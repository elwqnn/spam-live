<script lang="ts">
	import { page } from '$app/state';
	import { href } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
</script>

<svelte:head>
	<title>{page.status} · {m.site_name()}</title>
</svelte:head>

<div class="grid justify-items-center gap-3 px-4 py-16 text-center sm:px-6">
	<p class="text-5xl tabular-nums">{page.status}</p>
	<h1 class="font-heading text-3xl font-normal tracking-tight">
		{page.status === 404 ? m.error_not_found() : m.error_generic()}
	</h1>
	<a class="eyebrow text-muted-foreground transition-colors hover:text-foreground" href={href('/')}
		>{m.back_home()}</a
	>
</div>
