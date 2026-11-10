<script lang="ts">
	import type { Snippet } from 'svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowRight02Icon, Download04Icon, GithubIcon } from '@hugeicons/core-free-icons';
	import { formatDate, href, type Schemas } from '#lib';
	import TitleRow from '@spam-live/ui/components/TitleRow.svelte';
	import { Button } from '@spam-live/ui/components/ui/button/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import { getLocale } from '#lib/paraglide/runtime.js';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const PROFILE = 'https://github.com/elwqnn';
	const REPO = `${PROFILE}/spam-live`;
	const mb = (size: number) =>
		new Intl.NumberFormat(getLocale(), { maximumFractionDigits: 1 }).format(size / 1e6);
	const shell = (name: string) =>
		/^[\w.+-]+$/.test(name) ? name : `'${name.replaceAll("'", `'\\''`)}'`;
	const steps = [m.download_step_install, m.download_step_link, m.download_step_play];
	const link = 'underline-offset-4 transition-colors hover:text-foreground hover:underline';
</script>

<svelte:head>
	<title>{m.download_title()} · {m.site_name()}</title>
</svelte:head>

{#snippet block(
	os: string,
	arch: string,
	asset: Schemas['Asset'],
	primary: boolean,
	note: Snippet<[Schemas['Asset']]>
)}
	<section class="grid gap-3 p-4 sm:p-5">
		<div class="flex flex-wrap items-center gap-x-4 gap-y-3">
			<div class="grid gap-0.5">
				<h3 class="text-[15px] font-semibold">{os}</h3>
				<span class="text-xs text-muted-foreground">{arch}</span>
			</div>
			<div class="ml-auto flex items-center gap-3">
				<span class="text-xs text-muted-foreground tabular-nums"
					>{m.download_size({ size: mb(asset.size) })}</span
				>
				<Button
					href={asset.url}
					rel="external"
					variant={primary ? 'default' : 'outline'}
					aria-label={m.download_for({ os })}
					><HugeiconsIcon icon={Download04Icon} />{m.nav_download()}</Button
				>
			</div>
		</div>
		<div class="text-sm text-muted-foreground">{@render note(asset)}</div>
	</section>
{/snippet}

{#snippet windowsNote()}
	<p>{m.download_windows_note()}</p>
{/snippet}

{#snippet linuxNote(asset: Schemas['Asset'])}
	<p>{m.download_linux_note()}</p>
	<code class="mt-1.5 block font-mono text-xs wrap-anywhere text-foreground"
		>chmod +x {shell(asset.name)}</code
	>
{/snippet}

<TitleRow class="grid gap-2 row *:max-w-2xl">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
		{m.download_title()}
	</h1>
	<p class="text-muted-foreground">{m.download_intro()}</p>
</TitleRow>

<div
	class="grid flex-1 divide-y max-lg:content-start lg:grid-cols-[minmax(0,1fr)_22rem] lg:divide-x lg:divide-y-0"
>
	<div class="grid content-start gap-6 row sm:grid-cols-[5rem_minmax(0,1fr)] sm:gap-8">
		<img src="/desktop-icon.png" alt="" width="80" height="80" class="size-16 sm:size-20" />
		<div class="grid max-w-2xl min-w-0 content-start gap-6">
			{#if data.release}
				{@const release = data.release}
				<div class="grid gap-2">
					<h2 class="font-heading text-3xl font-normal tracking-tight tabular-nums sm:text-4xl">
						{m.download_version({ version: release.version })}
					</h2>
					<div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-muted-foreground">
						<time datetime={release.publishedAt}>{formatDate(release.publishedAt)}</time>
						<a class={link} href={release.url} rel="external"
							>{m.download_notes()}
							<HugeiconsIcon
								icon={ArrowRight02Icon}
								aria-hidden="true"
								class="inline size-[1em] align-[-0.125em]"
							/></a
						>
					</div>
				</div>
				{#if release.windows || release.linux}
					<div class="divide-y border bg-card">
						{#if release.windows}
							{@render block(
								'Windows',
								m.download_windows_arch(),
								release.windows,
								true,
								windowsNote
							)}
						{/if}
						{#if release.linux}
							{@render block('Linux', m.download_linux_arch(), release.linux, false, linuxNote)}
						{/if}
					</div>
				{/if}
			{:else}
				<div class="grid gap-2">
					<h2 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
						{data.missing ? m.download_none() : m.download_error()}
					</h2>
					<a class="text-sm text-muted-foreground {link}" href="{REPO}/releases" rel="external"
						>{m.download_releases()}
						<HugeiconsIcon
							icon={ArrowRight02Icon}
							aria-hidden="true"
							class="inline size-[1em] align-[-0.125em]"
						/></a
					>
				</div>
			{/if}
		</div>
	</div>

	<aside class="grid content-start gap-5 row">
		<section class="grid gap-3 border-b pb-5">
			<h2 class="eyebrow text-muted-foreground">{m.download_author()}</h2>
			<a class="group flex items-center gap-3" href={PROFILE} rel="external">
				<img
					src="{PROFILE}.png?size=96"
					alt=""
					width="48"
					height="48"
					class="size-12 shrink-0 rounded-full border"
				/>
				<span class="grid gap-0.5">
					<span class="font-medium underline-offset-4 group-hover:underline">elwqnn</span>
					<span class="flex items-center gap-1.5 text-sm text-muted-foreground"
						><HugeiconsIcon icon={GithubIcon} aria-hidden="true" class="size-4" />GitHub</span
					>
				</span>
			</a>
			<p class="text-sm leading-6 text-muted-foreground">{m.download_about()}</p>
		</section>
		<h2 class="font-heading text-2xl font-normal tracking-tight">{m.download_how()}</h2>
		<ol class="grid gap-3 text-sm">
			{#each steps as step, i (i)}
				<li class="flex gap-3">
					<span aria-hidden="true" class="text-muted-foreground tabular-nums"
						>{String(i + 1).padStart(2, '0')}</span
					>
					<span>{step()}</span>
				</li>
			{/each}
		</ol>
		<ul class="grid gap-2 border-t pt-5 text-sm text-muted-foreground">
			<li><a class={link} href={href('/import')}>{m.download_manual()}</a></li>
			<li><a class={link} href={href('/privacy')}>{m.footer_privacy()}</a></li>
			<li>
				<a class={link} href={REPO} rel="external"
					>{m.download_source()}
					<HugeiconsIcon
						icon={ArrowRight02Icon}
						aria-hidden="true"
						class="inline size-[1em] align-[-0.125em]"
					/></a
				>
			</li>
		</ul>
	</aside>
</div>
