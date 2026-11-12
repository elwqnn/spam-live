<script lang="ts">
	import type { Snippet } from 'svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { CloudIcon, CloudOffIcon } from '@hugeicons/core-free-icons';
	import type { Schemas } from '#ui';
	import { href } from '$host';
	import { m } from '$paraglide/messages.js';

	let {
		me,
		logout,
		status,
		children
	}: {
		me: Schemas['Me'];
		logout: () => void;
		status?: { live: boolean; label: string; href: string };
		children?: Snippet<[string, () => void]>;
	} = $props();

	let menu = $state<HTMLElement>();

	const uid = $props.id();
	const id = `${uid}-menu`;
	const initial = $derived(([...me.displayName.trim()][0] ?? '?').toUpperCase());
	const close = () => menu?.hidePopover();
	const item =
		'flex w-full cursor-pointer items-center gap-2 px-2 py-1.5 text-left text-sm transition-colors hover:bg-muted focus-visible:bg-muted focus-visible:outline-none';
</script>

{#snippet signal(live: boolean, className: string)}
	<HugeiconsIcon
		icon={CloudIcon}
		altIcon={CloudOffIcon}
		showAlt={!live}
		strokeWidth={2}
		aria-hidden="true"
		class={`${className} ${live ? 'text-up' : 'text-destructive'}`}
	/>
{/snippet}

<button
	type="button"
	popovertarget={id}
	data-private
	class="relative flex-none cursor-pointer rounded-full transition-colors outline-none hover:ring-2 hover:ring-foreground/25 focus-visible:ring-2 focus-visible:ring-ring"
	style:anchor-name="--{id}"
>
	<span class="sr-only">{m.account_menu()}{status ? `, ${status.label}` : ''}</span>
	{#if me.avatarUrl}
		<img src={me.avatarUrl} alt="" class="size-8 rounded-full" />
	{:else}
		<span
			aria-hidden="true"
			class="flex size-8 items-center justify-center rounded-full bg-muted text-xs font-medium text-muted-foreground"
			>{initial}</span
		>
	{/if}
	{#if status}
		<span
			class="absolute -right-1 -bottom-1 flex size-4 items-center justify-center rounded-full bg-background"
			>{@render signal(status.live, 'size-3')}</span
		>
	{/if}
</button>
<div
	bind:this={menu}
	popover
	{id}
	aria-label={m.account_menu()}
	class="w-56 max-w-[calc(100vw-1rem)] rounded-sm border bg-popover p-1 text-popover-foreground [position-area:block-end_span-inline-start] [position-try-fallbacks:flip-inline,flip-block,flip-block_flip-inline] supports-[position-area:block-end]:m-0 supports-[position-area:block-end]:my-1"
	style:position-anchor="--{id}"
>
	<div class="px-2 py-1.5">
		<p class="eyebrow text-muted-foreground">{m.signed_in_as()}</p>
		<p class="truncate text-sm font-semibold" data-private>{me.displayName}</p>
	</div>
	{#if status || children}
		<div class="mt-1 border-t pt-1">
			{#if status}
				<a class={item} href={status.href} onclick={close}
					>{@render signal(status.live, 'size-4 flex-none')}{status.label}</a
				>
			{/if}
			{@render children?.(item, close)}
		</div>
	{/if}
	<div class="mt-1 border-t pt-1">
		<a class={item} href={href('/me')} onclick={close}>{m.me_title()}</a>
		{#if me.isAdmin}
			<a class={item} href={href('/admin')} onclick={close}>{m.nav_admin()}</a>
		{/if}
		<button type="button" class={item} onclick={logout}>{m.logout()}</button>
	</div>
</div>
