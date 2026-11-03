<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import { loginHref } from '#lib';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Input } from '#lib/components/ui/input/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();
</script>

<svelte:head>
	<title>{m.link_title()} · {m.site_name()}</title>
</svelte:head>

<div class="grid max-w-md gap-4">
	<div class="grid gap-2">
		<h1 class="font-heading text-2xl font-semibold">{m.link_title()}</h1>
		<p class="text-sm text-muted-foreground">{m.link_text()}</p>
	</div>

	{#if form && !form.error}
		<p role="status" class="rounded-xl border p-3 text-sm">{m.link_done()}</p>
	{:else if data.me}
		<form method="POST" use:enhance class="grid gap-3">
			<label class="grid gap-1.5 text-sm">
				<span class="font-medium">{m.link_code()}</span>
				<Input
					name="userCode"
					required
					autocomplete="off"
					autocapitalize="characters"
					spellcheck="false"
					placeholder="XXXX-XXXX"
					value={page.url.searchParams.get('code') ?? ''}
					class="font-mono uppercase"
				/>
			</label>
			{#if form?.error}<p role="alert" class="text-sm text-destructive">{form.error}</p>{/if}
			<Button type="submit" class="justify-self-start">{m.link_submit()}</Button>
		</form>
	{:else}
		<p class="text-sm">{m.link_login()}</p>
		<Button href={loginHref(page.url)} rel="external" class="justify-self-start"
			>{m.login_steam()}</Button
		>
	{/if}
</div>
