<script lang="ts">
	import { href } from '#lib';
	import TitleRow from '#lib/components/TitleRow.svelte';
	import { m } from '#lib/paraglide/messages.js';

	const stored = [
		m.privacy_store_steam,
		m.privacy_store_account,
		m.privacy_store_matches,
		m.privacy_store_replays,
		m.privacy_store_content,
		m.privacy_store_devices,
		m.privacy_store_presence,
		m.privacy_store_app_presence,
		m.privacy_store_upload_source
	];
	const notStored = [
		m.privacy_not_store_opponent,
		m.privacy_not_store_tickets,
		m.privacy_not_store_logs,
		m.privacy_not_store_files
	];
	const retention = [
		m.privacy_retention_replays,
		m.privacy_retention_sessions,
		m.privacy_retention_rest
	];
</script>

<svelte:head>
	<title>{m.privacy_title()} · {m.site_name()}</title>
</svelte:head>

{#snippet list(items: (() => string)[])}
	<ul class="grid list-disc gap-1 pl-5">
		{#each items as item, i (i)}<li>{item()}</li>{/each}
	</ul>
{/snippet}

<article class="rows text-sm leading-6">
	<TitleRow class="grid gap-2 row *:max-w-2xl">
		<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
			{m.privacy_title()}
		</h1>
		<p class="text-muted-foreground">{m.privacy_intro()}</p>
	</TitleRow>

	<section class="grid gap-2 row *:max-w-2xl">
		<h2 class="font-heading text-2xl font-normal tracking-tight">{m.privacy_who_title()}</h2>
		<p>{m.privacy_who_text()}</p>
		<p>{m.privacy_contact()}</p>
	</section>

	<section class="grid gap-2 row *:max-w-2xl">
		<h2 class="font-heading text-2xl font-normal tracking-tight">{m.privacy_store_title()}</h2>
		{@render list(stored)}
	</section>

	<section class="grid gap-2 row *:max-w-2xl">
		<h2 class="font-heading text-2xl font-normal tracking-tight">
			{m.privacy_not_store_title()}
		</h2>
		{@render list(notStored)}
	</section>

	<section class="grid gap-2 row *:max-w-2xl">
		<h2 class="font-heading text-2xl font-normal tracking-tight">{m.privacy_why_title()}</h2>
		<p>{m.privacy_why_text()}</p>
	</section>

	<section class="grid gap-2 row *:max-w-2xl">
		<h2 class="font-heading text-2xl font-normal tracking-tight">
			{m.privacy_retention_title()}
		</h2>
		{@render list(retention)}
	</section>

	<section class="grid gap-2 row *:max-w-2xl">
		<h2 class="font-heading text-2xl font-normal tracking-tight">{m.privacy_delete_title()}</h2>
		<p>
			{m.privacy_delete_text()}
			<a class="underline underline-offset-4" href={href('/me')}>{m.me_title()}</a>.
			{m.privacy_delete_all()}
		</p>
		{@render list([m.privacy_delete_matches, m.privacy_delete_ban])}
	</section>

	<section class="grid gap-2 row *:max-w-2xl">
		<h2 class="font-heading text-2xl font-normal tracking-tight">{m.privacy_cookies_title()}</h2>
		<p>{m.privacy_cookies_text()}</p>
	</section>
</article>
