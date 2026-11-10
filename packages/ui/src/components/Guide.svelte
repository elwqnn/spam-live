<script lang="ts">
	let { html }: { html: string } = $props();
</script>

<div class="guide text-sm leading-6 break-words">{@html html}</div>

<style>
	.guide :global(:where(h1, h2, h3, h4, h5, h6)) {
		margin: 1.25em 0 0.5em;
		font-weight: 600;
		line-height: 1.3;
	}
	.guide :global(h1) {
		font-size: 1.25rem;
	}
	.guide :global(h2) {
		font-size: 1.125rem;
	}
	.guide :global(h3) {
		font-size: 1rem;
	}
	.guide :global(:where(p, ul, ol, blockquote, pre, table)) {
		margin: 0.75em 0;
	}
	.guide :global(> :first-child) {
		margin-top: 0;
	}
	.guide :global(ul) {
		list-style: disc;
		padding-left: 1.5em;
	}
	.guide :global(ol) {
		list-style: decimal;
		padding-left: 1.5em;
	}
	.guide :global(a) {
		text-decoration: underline;
		text-underline-offset: 4px;
	}
	.guide :global(code) {
		border-radius: 2px;
		background: var(--muted);
		padding: 0.1em 0.3em;
		font-size: 0.875em;
	}
	.guide :global(pre) {
		overflow-x: auto;
		background: var(--muted);
		padding: 0.75em 1em;
	}
	.guide :global(pre code) {
		background: none;
		padding: 0;
	}
	.guide :global(blockquote) {
		border-left: 3px solid var(--border);
		padding-left: 1em;
		color: var(--muted-foreground);
	}
	.guide :global(table) {
		display: block;
		overflow-x: auto;
		border-collapse: collapse;
	}
	.guide :global(:where(th, td)) {
		border: 1px solid var(--border);
		padding: 0.25em 0.5em;
	}
	.guide :global(hr) {
		margin: 1.5em 0;
	}
</style>
