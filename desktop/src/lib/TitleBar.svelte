<script lang="ts">
	import { onMount } from 'svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Cancel01Icon, Copy01Icon, MinusSignIcon, SquareIcon } from '@hugeicons/core-free-icons';
	import { getVersion } from '@tauri-apps/api/app';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import Logo from '@spam-live/ui/components/Logo.svelte';
	import { isMac } from '@spam-live/ui/shortcut.js';
	import { m } from '#lib/paraglide/messages.js';

	const win = getCurrentWindow();
	const mac = isMac();
	const base =
		'flex w-11 items-center justify-center text-muted-foreground transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset [&_svg]:size-4';
	const button = `${base} hover:bg-muted hover:text-foreground`;
	let maximized = $state(false);
	let version = $state('');

	onMount(() => {
		document.documentElement.classList.add('framed');
		getVersion()
			.then((v) => (version = v))
			.catch(() => undefined);
		if (mac) return;
		const sync = () => win.isMaximized().then((on) => (maximized = on));
		sync();
		const off = win.onResized(sync);
		return () => off.then((stop) => stop());
	});
</script>

<header
	data-tauri-drag-region="deep"
	class={['sticky top-0 z-40 flex h-(--head) border-b bg-background select-none', mac && 'pl-20']}
>
	<div class="flex items-center gap-2 px-3">
		<Logo class="size-4" />
		<span class="text-sm font-semibold tracking-wide"
			>SPAM<span class="text-muted-foreground">.live</span></span
		>
		{#if version}<span class="text-xs text-muted-foreground tabular-nums">{version}</span>{/if}
	</div>
	{#if !mac}
		<div class="ml-auto flex">
			<button
				type="button"
				class={button}
				aria-label={m.desktop_window_minimize()}
				onclick={() => win.minimize()}
				><HugeiconsIcon icon={MinusSignIcon} aria-hidden="true" /></button
			>
			<button
				type="button"
				class={button}
				aria-label={maximized ? m.desktop_window_restore() : m.desktop_window_maximize()}
				onclick={() => win.toggleMaximize()}
				><HugeiconsIcon
					icon={SquareIcon}
					altIcon={Copy01Icon}
					showAlt={maximized}
					aria-hidden="true"
				/></button
			>
			<button
				type="button"
				class={`${base} hover:bg-destructive hover:text-background`}
				aria-label={m.desktop_window_close()}
				onclick={() => win.close()}><HugeiconsIcon icon={Cancel01Icon} aria-hidden="true" /></button
			>
		</div>
	{/if}
</header>
