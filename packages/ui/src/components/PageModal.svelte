<script lang="ts">
	import { untrack, type Snippet } from 'svelte';
	import type { Attachment } from 'svelte/attachments';
	import { goto, preloadData, pushState, replaceState } from '$app/navigation';
	import { page } from '$app/state';
	import { Button } from '#ui/components/ui/button/index.js';
	import { modalKind, pageUrl, type Modal } from '#ui/modal.js';
	import { m } from '$paraglide/messages.js';
	import { close as closePreview } from './CardPreview.svelte';

	let { children }: { children: Snippet<[Modal]> } = $props();

	const PAGES = ['/cards/[id]', '/decks/[id]', '/decks/[id]/edit', '/decks/new'];

	let kept: Modal | undefined;
	const modal = $derived.by(() => {
		kept = page.state.modal ?? (page.shallow ? kept : undefined);
		return kept;
	});
	const href = $derived(modal?.href);
	const shown = $derived(!!modal);
	let clicks = 0;
	let leaving = false;

	function back() {
		if (leaving) return;
		leaving = true;
		history.back();
	}

	function reveal(node: Element | null, hash: string) {
		const id = decodeURIComponent(hash.slice(1));
		if (id) node?.querySelector(`#${CSS.escape(id)}`)?.scrollIntoView();
	}

	async function intercept(e: MouseEvent) {
		const a = e.target instanceof Element ? e.target.closest('a[href]') : null;
		if (!a || e.defaultPrevented || e.button || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey)
			return;
		if (a.closest('[data-sveltekit-reload]')) return;
		const target = a.getAttribute('target');
		const rel = a.getAttribute('rel')?.split(/\s+/) ?? [];
		if ((target && target !== '_self') || a.hasAttribute('download') || rel.includes('external'))
			return;
		const url = new URL(a.getAttribute('href') ?? '', location.href);
		const kind = modalKind(url.pathname);
		if (url.origin !== location.origin || !kind) return;
		const here = pageUrl();
		if (url.pathname + url.search === here.pathname + here.search) {
			if (!modal || !url.hash) return;
			e.preventDefault();
			reveal(document.getElementById('page-modal'), url.hash);
			return;
		}
		if (!modal && PAGES.includes(page.route.id ?? '')) return;
		e.preventDefault();
		const id = ++clicks;
		const result = await preloadData(url.href);
		if (id !== clicks) return;
		if (result.type === 'loaded' && result.status === 200)
			pushState(url.href, { modal: { kind, href: url.href, data: result.data } });
		else goto(url.href);
	}

	async function refresh(current: Modal) {
		const result = await preloadData(current.href).catch(() => null);
		if (modal?.href !== current.href) return;
		const fresh = result?.type === 'loaded' && result.status === 200;
		replaceState(current.href, { modal: fresh ? { ...current, data: result.data } : current });
	}

	$effect(() => {
		const current = page.data && untrack(() => modal);
		if (current) refresh(current);
	});

	$effect.pre(() => {
		if (!shown) return;
		const title = document.title;
		return () => (document.title = title);
	});

	const show: Attachment<HTMLDialogElement> = (node) => {
		leaving = false;
		if (!href) return;
		closePreview();
		if (!node.open) node.showModal();
		const h1 = node.querySelector('h1');
		if (h1) {
			h1.id ||= 'page-modal-title';
			node.setAttribute('aria-labelledby', h1.id);
		} else node.removeAttribute('aria-labelledby');
		node.scrollTo(0, 0);
		reveal(node, new URL(href).hash);
	};

	function dismiss(e: MouseEvent & { currentTarget: HTMLDialogElement }) {
		const r = e.currentTarget.getBoundingClientRect();
		const out =
			e.clientX < r.left || e.clientX > r.right || e.clientY < r.top || e.clientY > r.bottom;
		if (e.target === e.currentTarget && out) back();
	}
</script>

<svelte:body onclick={intercept} />
<svelte:window onpopstate={() => (leaving = false)} />

{#if modal}
	<dialog
		id="page-modal"
		closedby="any"
		oncancel={(e) => {
			if (!e.cancelable) return;
			e.preventDefault();
			back();
		}}
		onclose={() => {
			if (modal) back();
		}}
		onclick={dismiss}
		{@attach show}
		class="m-auto max-h-[calc(100dvh-2rem)] w-[min(100%-2rem,80rem)] max-w-none scroll-pt-16 overflow-x-hidden overflow-y-auto border bg-background p-0 text-foreground backdrop:bg-black/70 motion-safe:animate-in motion-safe:zoom-in-95 motion-safe:fade-in motion-safe:backdrop:animate-in motion-safe:backdrop:fade-in"
	>
		<div class="sticky top-0 z-20 flex justify-end border-b bg-background px-(--gutter) py-3">
			<Button variant="outline" size="sm" onclick={back}>{m.close()}</Button>
		</div>
		<div class="rows flex flex-col">
			{#key modal.href}
				{@render children(modal)}
			{/key}
		</div>
	</dialog>
{/if}

<style>
	:global(html:has(#page-modal[open])) {
		overflow: hidden;
		scrollbar-gutter: stable;
	}
</style>
