<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import { loginHref } from '#lib';
	import { Button } from '@spam-live/ui/components/ui/button/index.js';
	import { Input } from '@spam-live/ui/components/ui/input/index.js';
	import TitleRow from '@spam-live/ui/components/TitleRow.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();
</script>

<svelte:head>
	<title>{m.link_title()} · {m.site_name()}</title>
</svelte:head>

<TitleRow class="grid gap-2 row *:max-w-md">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">{m.link_title()}</h1>
	<p class="text-sm text-muted-foreground">{m.link_text()}</p>
</TitleRow>

{#if form && !form.error}
	<p role="status" class="row text-sm">{m.link_done()}</p>
{:else if data.me}
	<form method="POST" use:enhance class="grid gap-3 row *:max-w-md">
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
	<div class="grid gap-3 row">
		<p class="text-sm">{m.link_login()}</p>
		<Button href={loginHref(page.url)} rel="external" class="justify-self-start"
			>{m.login_steam()}</Button
		>
	</div>
{/if}
