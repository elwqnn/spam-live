<script lang="ts">
	import { HugeiconsIcon, type IconSvgElement } from '@hugeicons/svelte';
	import {
		BubbleChatIcon,
		Bug01Icon,
		BulbIcon,
		Camera01Icon,
		Cancel01Icon,
		CrosshairIcon,
		PencilEdit02Icon,
		RefreshIcon,
		SentIcon,
		SquareDashedMousePointerIcon
	} from '@hugeicons/core-free-icons';
	import { page } from '$app/state';
	import { loginHref, type Schemas } from '#lib';
	import {
		KINDS,
		elementText,
		pagePath,
		selector,
		siteVersion,
		validBody,
		type Kind
	} from '#lib/feedback.js';
	import Dialog from '#lib/components/Dialog.svelte';
	import Tooltip from '#lib/components/Tooltip.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { Textarea } from '#lib/components/ui/textarea/index.js';
	import { m } from '#lib/paraglide/messages.js';
	import Annotator from './Annotator.svelte';
	import Picker from './Picker.svelte';
	import { capture, encode, flatten, frame, type Base, type Mark } from './capture.js';

	let { me }: { me: Schemas['Me'] | null } = $props();

	const ID = 'feedback';
	const KIND: Record<Kind, [IconSvgElement, () => string, () => string]> = {
		bug: [Bug01Icon, m.feedback_bug, m.feedback_placeholder_bug],
		idea: [BulbIcon, m.feedback_idea, m.feedback_placeholder_idea],
		other: [BubbleChatIcon, m.feedback_other, m.feedback_placeholder_other]
	};

	type Shot = { blob: Blob; url: string; width: number; height: number };
	type Status = 'idle' | 'sending' | 'sent' | 'limit' | 'error';

	let kind = $state<Kind>('bug');
	let text = $state('');
	let base = $state.raw<Base | null>(null);
	let marks = $state.raw<Mark[]>([]);
	let shot = $state.raw<Shot | null>(null);
	let picked = $state.raw<{ selector: string; text: string } | null>(null);
	let status = $state<Status>('idle');
	let mode = $state<'annotate' | 'pick' | null>(null);
	let busy = false;
	let run = 0;
	let closing: ReturnType<typeof setTimeout> | undefined;

	const valid = $derived(validBody(text));
	const dialog = () => document.getElementById(ID) as HTMLDialogElement | null;
	const show = () => dialog()?.showModal();
	const hide = () => dialog()?.close();

	function setShot(next: Shot | null) {
		if (shot) URL.revokeObjectURL(shot.url);
		shot = next;
	}

	const keep = (result: Omit<Shot, 'url'>) =>
		setShot({ ...result, url: URL.createObjectURL(result.blob) });

	async function shoot() {
		const source = await capture();
		return { source, result: await encode(flatten(source, [])) };
	}

	async function take() {
		const id = ++run;
		busy = true;
		remove();
		await frame();
		await frame();
		let timer: ReturnType<typeof setTimeout> | undefined;
		const limit = new Promise<never>((_, reject) => {
			timer = setTimeout(() => reject(new Error('timeout')), 8000);
		});
		try {
			const { source, result } = await Promise.race([shoot(), limit]);
			if (id !== run) return;
			base = source;
			keep(result);
		} catch {
			return;
		} finally {
			clearTimeout(timer);
			busy = false;
		}
	}

	export async function open() {
		if (busy || mode || dialog()?.open) return;
		clearTimeout(closing);
		if (status === 'sent') reset();
		if (me) {
			picked = null;
			await take();
		}
		show();
	}

	async function retake() {
		hide();
		await take();
		show();
	}

	function remove() {
		base = null;
		marks = [];
		setShot(null);
	}

	function annotate() {
		hide();
		mode = 'annotate';
	}

	async function annotated(next: Mark[]) {
		mode = null;
		const source = base;
		if (source) {
			const id = ++run;
			busy = true;
			marks = next;
			try {
				const result = await encode(flatten(source, next));
				if (id === run) keep(result);
			} catch {
				if (id === run) remove();
			} finally {
				busy = false;
			}
		}
		show();
	}

	function pick(el: Element) {
		mode = null;
		picked = {
			selector: selector(el, (id) => document.querySelectorAll(`#${CSS.escape(id)}`).length === 1),
			text: elementText(el)
		};
		show();
	}

	function back() {
		mode = null;
		show();
	}

	function reset() {
		run++;
		kind = 'bug';
		text = '';
		remove();
		picked = null;
		status = 'idle';
	}

	async function send(e: SubmitEvent) {
		e.preventDefault();
		if (!valid || status === 'sending' || status === 'sent') return;
		status = 'sending';
		const form = new FormData();
		form.set('kind', kind);
		form.set('body', text.trim());
		form.set('page', pagePath(location.pathname, location.search));
		form.set('siteVersion', siteVersion(import.meta.env.VITE_SITE_VERSION));
		if (picked) form.set('element', JSON.stringify(picked));
		if (shot)
			form.set(
				'screenshot',
				shot.blob,
				`screenshot.${shot.blob.type === 'image/png' ? 'png' : 'webp'}`
			);
		try {
			const res = await fetch('/api/feedback', { method: 'POST', body: form });
			status = res.ok ? 'sent' : res.status === 429 ? 'limit' : 'error';
		} catch {
			status = 'error';
		}
		if (status !== 'sent') return;
		closing = setTimeout(() => {
			hide();
			reset();
		}, 1500);
	}
</script>

{#snippet action(icon: IconSvgElement, label: string, onclick: () => void)}
	<Tooltip content={label}>
		{#snippet children(tip)}
			<Button variant="ghost" size="icon-sm" aria-label={label} {onclick} {...tip}
				><HugeiconsIcon {icon} /></Button
			>
		{/snippet}
	</Tooltip>
{/snippet}

<Dialog id={ID} title={m.feedback_title()}>
	<Button
		variant="ghost"
		size="icon-sm"
		class="absolute top-6 right-5"
		aria-label={m.close()}
		commandfor={ID}
		command="close"><HugeiconsIcon icon={Cancel01Icon} /></Button
	>
	{#if !me}
		<a class="text-sm underline underline-offset-4" href={loginHref(page.url)} rel="external"
			>{m.feedback_login()}</a
		>
	{:else if status === 'sent'}
		<p role="status" class="text-sm">{m.feedback_thanks()}</p>
	{:else}
		<form class="grid gap-4" onsubmit={send}>
			<p class="-mt-1 text-sm text-muted-foreground">{m.feedback_intro()}</p>
			<div
				role="group"
				aria-label={m.feedback_kind()}
				class="grid grid-cols-3 overflow-hidden rounded-sm border border-foreground/25"
			>
				{#each KINDS as key (key)}
					{@const [icon, label] = KIND[key]}
					<Button
						variant={kind === key ? 'secondary' : 'ghost'}
						size="sm"
						class="rounded-none border-0 focus-visible:ring-inset {kind === key
							? ''
							: 'text-muted-foreground'}"
						aria-pressed={kind === key}
						onclick={() => (kind = key)}><HugeiconsIcon {icon} />{label()}</Button
					>
				{/each}
			</div>
			<Textarea
				bind:value={text}
				aria-label={m.feedback_text_label()}
				placeholder={KIND[kind][2]()}
				maxlength={5000}
				class="max-h-64 min-h-28"
			/>
			<div
				role="group"
				aria-label={m.feedback_attachments()}
				class="grid divide-y rounded-sm border bg-card text-sm"
			>
				<div class="flex items-center gap-2.5 p-2.5 sm:gap-3 sm:p-3">
					{#if shot}
						<img
							src={shot.url}
							alt=""
							class="h-10 w-16 shrink-0 rounded-xs border object-cover object-top sm:h-14 sm:w-24"
						/>
					{:else}
						<span
							class="flex h-10 w-16 shrink-0 items-center justify-center rounded-xs border border-dashed text-muted-foreground sm:h-14 sm:w-24"
							aria-hidden="true"><HugeiconsIcon icon={Camera01Icon} class="size-4" /></span
						>
					{/if}
					<div class="grid min-w-0 flex-1 gap-0.5">
						<span class="font-medium">{m.feedback_screenshot()}</span>
						<span class="text-xs text-muted-foreground tabular-nums">
							{#if shot}{shot.width} × {shot.height} · {m.feedback_marks({
									count: marks.length
								})}{:else}{m.feedback_no_screenshot()}{/if}
						</span>
					</div>
					<div class="flex shrink-0">
						{#if shot}
							{@render action(PencilEdit02Icon, m.feedback_annotate(), annotate)}
							{@render action(RefreshIcon, m.feedback_retake(), retake)}
							{@render action(Cancel01Icon, m.feedback_remove(), remove)}
						{:else}
							{@render action(Camera01Icon, m.feedback_capture(), retake)}
						{/if}
					</div>
				</div>
				<div class="flex items-center gap-2.5 p-2.5 sm:gap-3 sm:p-3">
					<span
						class="flex h-10 w-16 shrink-0 items-center justify-center rounded-xs border border-dashed text-muted-foreground sm:h-14 sm:w-24"
						aria-hidden="true"
						><HugeiconsIcon icon={SquareDashedMousePointerIcon} class="size-4" /></span
					>
					<div class="grid min-w-0 flex-1 gap-0.5">
						<span class="font-medium">{m.feedback_element()}</span>
						{#if picked}
							<code class="truncate text-xs text-muted-foreground">{picked.selector}</code>
						{:else}
							<span class="text-xs text-muted-foreground">{m.feedback_no_element()}</span>
						{/if}
					</div>
					<div class="flex shrink-0">
						{@render action(CrosshairIcon, m.feedback_pick(), () => {
							hide();
							mode = 'pick';
						})}
						{#if picked}
							{@render action(Cancel01Icon, m.feedback_clear_element(), () => (picked = null))}
						{/if}
					</div>
				</div>
			</div>
			<p class="text-xs leading-5 text-muted-foreground">{m.feedback_note()}</p>
			{#if status === 'limit'}
				<p role="alert" class="text-sm text-destructive">{m.feedback_limit()}</p>
			{:else if status === 'error'}
				<p role="alert" class="text-sm text-destructive">{m.feedback_error()}</p>
			{/if}
			<div class="flex justify-end">
				<Button type="submit" disabled={!valid || status === 'sending'}
					><HugeiconsIcon icon={SentIcon} />{status === 'sending'
						? m.feedback_sending()
						: m.feedback_title()}</Button
				>
			</div>
		</form>
	{/if}
</Dialog>

{#if mode === 'annotate' && base}
	<Annotator {base} {marks} ondone={annotated} oncancel={back} />
{:else if mode === 'pick'}
	<Picker onpick={pick} oncancel={back} />
{/if}
