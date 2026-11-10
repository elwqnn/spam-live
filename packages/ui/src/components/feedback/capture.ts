import { MAX_BYTES, blockSize, captureScale, pixelate } from '#ui/feedback.js';

export type Tool = 'pen' | 'rect' | 'arrow' | 'highlight' | 'blur';
export type Point = [number, number];
export type Mark = { tool: Tool; points: Point[] };
export type Base = { canvas: HTMLCanvasElement; scale: number };

const RED = '#ef4444';
const YELLOW = 'rgba(250, 204, 21, 0.35)';
const STICKY = 'data-shot-sticky';
const BOX = 'data-shot-box';
const MARGIN = 100;
const QUALITIES = [0.9, 0.75, 0.6, 0.45];

export const frame = () => new Promise((resolve) => requestAnimationFrame(resolve));

function contained(el: Element) {
	const style = getComputedStyle(el);
	const parent = el.parentElement ? getComputedStyle(el.parentElement).display : '';
	return (
		/flex|grid/.test(parent) ||
		style.overflow !== 'visible' ||
		/flow-root|inline-block|table-cell/.test(style.display) ||
		style.position === 'absolute'
	);
}

function prepare(width: number, height: number) {
	const pinned: Element[] = [];
	const marked: Element[] = [];
	for (const el of document.body.querySelectorAll<HTMLElement>('*')) {
		const position = getComputedStyle(el).position;
		if (position === 'fixed') pinned.push(el);
		if (position !== 'sticky') continue;
		pinned.push(el);
		const stuck = el.getBoundingClientRect();
		const value = el.style.getPropertyValue('position');
		const priority = el.style.getPropertyPriority('position');
		el.style.setProperty('position', 'static', 'important');
		const flow = el.getBoundingClientRect();
		el.style.setProperty('position', value, priority);
		const dx = stuck.left - flow.left;
		const dy = stuck.top - flow.top;
		if (Math.abs(dx) < 0.5 && Math.abs(dy) < 0.5) continue;
		el.setAttribute(STICKY, `${dx} ${dy}`);
		marked.push(el);
	}
	const visit = (el: Element) => {
		for (const child of el.children) {
			const r = child.getBoundingClientRect();
			const below = r.top > height + MARGIN;
			const out = below || r.bottom < -MARGIN || r.right < -MARGIN || r.left > width + MARGIN;
			if (
				out &&
				(below || contained(child)) &&
				!pinned.some((p) => child.contains(p)) &&
				getComputedStyle(child).display !== 'contents'
			) {
				child.setAttribute(BOX, `${r.width} ${r.height}`);
				marked.push(child);
			} else visit(child);
		}
	};
	visit(document.body);
	return marked;
}

function fix(clone: Node) {
	if (!(clone instanceof HTMLElement)) return;
	const box = clone.getAttribute(BOX);
	if (box) {
		const [w, h] = box.split(' ');
		clone.style.setProperty('box-sizing', 'border-box');
		clone.style.setProperty('width', `${w}px`);
		clone.style.setProperty('height', `${h}px`);
		clone.style.setProperty('min-width', '0');
		clone.style.setProperty('min-height', '0');
		clone.style.setProperty('flex', 'none');
		clone.removeAttribute(BOX);
	}
	const sticky = clone.getAttribute(STICKY);
	if (sticky) {
		const [dx, dy] = sticky.split(' ');
		clone.style.setProperty('position', 'relative');
		clone.style.setProperty('inset', `${dy}px auto auto ${dx}px`);
		clone.removeAttribute(STICKY);
	}
}

function privateRects(width: number, height: number) {
	return [...document.querySelectorAll('[data-private]')]
		.filter((el) => el.checkVisibility?.() ?? true)
		.map((el) => el.getBoundingClientRect())
		.filter((r) => r.width && r.height && r.right > 0 && r.bottom > 0)
		.filter((r) => r.left < width && r.top < height);
}

let active = false;

export function pixelateRect(
	ctx: CanvasRenderingContext2D,
	x: number,
	y: number,
	w: number,
	h: number,
	block: number
) {
	const left = Math.max(0, Math.floor(x));
	const top = Math.max(0, Math.floor(y));
	const right = Math.min(ctx.canvas.width, Math.ceil(x + w));
	const bottom = Math.min(ctx.canvas.height, Math.ceil(y + h));
	if (right <= left || bottom <= top) return;
	const image = ctx.getImageData(left, top, right - left, bottom - top);
	pixelate(image.data, image.width, image.height, block);
	ctx.putImageData(image, left, top);
}

export async function capture(): Promise<Base> {
	if (active) throw new Error('busy');
	active = true;
	let marked: Element[] = [];
	try {
		const { createContext, domToCanvas } = await import('modern-screenshot');
		const width = document.documentElement.clientWidth;
		const height = document.documentElement.clientHeight;
		const scroll = window.scrollY;
		const popover = CSS.supports('selector(:popover-open)');
		marked = prepare(width, height);
		const rects = privateRects(width, height);
		const context = await createContext(document.body, {
			width,
			height,
			scale: captureScale(width, height, devicePixelRatio),
			backgroundColor: getComputedStyle(document.body).backgroundColor,
			timeout: 300,
			autoDestruct: true,
			style: { margin: `${-scroll}px 0 0`, height: `${scroll + height}px`, overflow: 'hidden' },
			filter: (node) =>
				!node.parentElement?.hasAttribute(BOX) &&
				!(
					node instanceof Element &&
					((popover && node.matches(':popover-open')) || node.matches('dialog[open]'))
				),
			onCloneEachNode: fix
		});
		context.timeout = 5000;
		const canvas = await domToCanvas(context);
		const ctx = canvas.getContext('2d');
		const sx = canvas.width / width;
		const sy = canvas.height / height;
		for (const r of rects)
			if (ctx)
				pixelateRect(ctx, r.left * sx, r.top * sy, r.width * sx, r.height * sy, blockSize(sx));
		return { canvas, scale: sx };
	} finally {
		for (const el of marked) {
			el.removeAttribute(BOX);
			el.removeAttribute(STICKY);
		}
		active = false;
	}
}

export const lineWidth = (base: Base) => Math.max(2, Math.round(3 * base.scale));

export function drawMark(ctx: CanvasRenderingContext2D, mark: Mark, base: Base) {
	const [a] = mark.points;
	const b = mark.points[mark.points.length - 1];
	const x = Math.min(a[0], b[0]);
	const y = Math.min(a[1], b[1]);
	const w = Math.abs(b[0] - a[0]);
	const h = Math.abs(b[1] - a[1]);
	const line = lineWidth(base);
	ctx.save();
	ctx.strokeStyle = RED;
	ctx.lineWidth = line;
	ctx.lineCap = 'round';
	ctx.lineJoin = 'round';
	if (mark.tool === 'pen') {
		ctx.beginPath();
		ctx.moveTo(...a);
		for (const p of mark.points) ctx.lineTo(...p);
		ctx.stroke();
	} else if (mark.tool === 'rect') {
		ctx.strokeRect(x, y, w, h);
	} else if (mark.tool === 'arrow') {
		const angle = Math.atan2(b[1] - a[1], b[0] - a[0]);
		const head = line * 5;
		ctx.beginPath();
		ctx.moveTo(...a);
		ctx.lineTo(...b);
		for (const turn of [-0.5, 0.5]) {
			ctx.moveTo(...b);
			ctx.lineTo(b[0] - head * Math.cos(angle + turn), b[1] - head * Math.sin(angle + turn));
		}
		ctx.stroke();
	} else if (mark.tool === 'highlight') {
		ctx.fillStyle = YELLOW;
		ctx.fillRect(x, y, w, h);
	} else {
		pixelateRect(ctx, x, y, w, h, blockSize(base.scale));
	}
	ctx.restore();
}

export function flatten(base: Base, marks: Mark[]) {
	const canvas = document.createElement('canvas');
	canvas.width = base.canvas.width;
	canvas.height = base.canvas.height;
	const ctx = canvas.getContext('2d');
	if (!ctx) return canvas;
	ctx.drawImage(base.canvas, 0, 0);
	for (const mark of marks) drawMark(ctx, mark, base);
	return canvas;
}

const toBlob = (canvas: HTMLCanvasElement, type: string, quality: number) =>
	new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, type, quality));

function shrink(canvas: HTMLCanvasElement) {
	const small = document.createElement('canvas');
	small.width = Math.max(1, Math.round(canvas.width * 0.75));
	small.height = Math.max(1, Math.round(canvas.height * 0.75));
	small.getContext('2d')?.drawImage(canvas, 0, 0, small.width, small.height);
	return small;
}

export async function encode(canvas: HTMLCanvasElement) {
	let type = 'image/webp';
	for (let c = canvas; c.width >= 16 && c.height >= 16; c = shrink(c)) {
		for (const quality of QUALITIES) {
			const blob = await toBlob(c, type, quality);
			if (!blob) throw new Error('encode');
			if (blob.type !== 'image/webp') type = 'image/png';
			if (blob.size <= MAX_BYTES) return { blob, width: c.width, height: c.height };
			if (type === 'image/png') break;
		}
	}
	throw new Error('encode');
}
