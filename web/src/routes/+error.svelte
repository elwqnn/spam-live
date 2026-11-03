<script lang="ts">
	import { page } from '$app/state';
	import { href } from '#lib';
	import { m } from '#lib/paraglide/messages.js';
</script>

<svelte:head>
	<title>{page.status} · {m.site_name()}</title>
</svelte:head>

<div class="grid gap-2 py-12 text-center">
	<p class="text-4xl font-semibold tabular-nums">{page.status}</p>
	<h1 class="text-lg">{page.status === 404 ? m.error_not_found() : m.error_generic()}</h1>
	<a class="text-sm text-muted-foreground hover:underline" href={href('/')}>{m.back_home()}</a>
</div>
