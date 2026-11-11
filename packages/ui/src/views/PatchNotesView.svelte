<script lang="ts">
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowRight02Icon, ArrowUpRight01Icon } from '@hugeicons/core-free-icons';
	import { href } from '$host';
	import { STEAM_STORE } from '#ui';
	import CardStack from '#ui/components/CardStack.svelte';
	import Time from '#ui/components/Time.svelte';
	import TitleRow from '#ui/components/TitleRow.svelte';
	import { m } from '$paraglide/messages.js';
	import type { patchNotesData } from './data.js';

	let { data }: { data: Awaited<ReturnType<typeof patchNotesData>> } = $props();

	const link = 'underline underline-offset-4 transition-colors hover:text-foreground';
	const icon = 'inline size-[1em] align-[-0.125em]';
</script>

<svelte:head>
	<title>{m.patch_notes_title()} · {m.site_name()}</title>
	<meta name="description" content={m.patch_notes_intro()} />
</svelte:head>

<TitleRow class="grid gap-2 row *:max-w-2xl">
	<h1 class="font-heading text-3xl font-normal tracking-tight sm:text-4xl">
		{m.patch_notes_title()}
	</h1>
	<p class="text-muted-foreground">{m.patch_notes_intro()}</p>
	<p class="text-sm text-muted-foreground">
		<a class={link} href={STEAM_STORE} target="_blank" rel="external noopener">{m.steam_data()}</a>
	</p>
</TitleRow>

{#if data.news.length}
	<CardStack>
		{#each data.news as item (item.gid)}
			<li class="grid gap-1 px-(--gutter) py-4 sm:grid-cols-[10rem_minmax(0,1fr)] sm:gap-6">
				<Time iso={item.publishedAt} class="pt-1 text-sm text-muted-foreground tabular-nums" />
				<div class="grid gap-1.5">
					<h2 class="font-heading text-xl font-normal tracking-tight">{item.title}</h2>
					<p class="flex flex-wrap gap-x-5 gap-y-1 text-sm text-muted-foreground">
						<a
							class={link}
							href={item.url}
							target="_blank"
							rel="external noopener"
							aria-label={m.patch_notes_read_label({ title: item.title })}
							>{m.patch_notes_read()}
							<HugeiconsIcon icon={ArrowUpRight01Icon} aria-hidden="true" class={icon} /></a
						>
						{#if item.version}
							<a class={link} href={href(`/stats?version=${encodeURIComponent(item.version)}`)}
								>{m.patch_notes_stats({ version: item.version })}
								<HugeiconsIcon icon={ArrowRight02Icon} aria-hidden="true" class={icon} /></a
							>
						{/if}
					</p>
				</div>
			</li>
		{/each}
	</CardStack>
{:else}
	<p class="row text-sm text-muted-foreground">{m.patch_notes_empty()}</p>
{/if}
