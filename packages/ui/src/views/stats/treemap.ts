export type Box = { x: number; y: number; w: number; h: number };

export function squarify<T>(items: T[], value: (item: T) => number, box: Box) {
	const out: (Box & { item: T })[] = [];
	const total = items.reduce((sum, item) => sum + value(item), 0);
	if (!(total > 0) || box.w <= 0 || box.h <= 0) return out;
	const scale = (box.w * box.h) / total;
	const queue = items
		.map((item) => ({ item, area: value(item) * scale }))
		.sort((a, b) => b.area - a.area);
	let { x, y, w, h } = box;
	let i = 0;
	while (i < queue.length) {
		const side = Math.min(w, h);
		const max = queue[i].area;
		let j = i;
		let sum = 0;
		let worst = Infinity;
		while (j < queue.length) {
			const next = sum + queue[j].area;
			const ratio = Math.max(
				(side * side * max) / (next * next),
				(next * next) / (side * side * queue[j].area)
			);
			if (ratio > worst) break;
			worst = ratio;
			sum = next;
			j++;
		}
		const thick = sum / side;
		let offset = 0;
		for (const { item, area } of queue.slice(i, j)) {
			const length = area / thick;
			out.push(
				w >= h
					? { item, x, y: y + offset, w: thick, h: length }
					: { item, x: x + offset, y, w: length, h: thick }
			);
			offset += length;
		}
		if (w >= h) {
			x += thick;
			w -= thick;
		} else {
			y += thick;
			h -= thick;
		}
		i = j;
	}
	return out;
}
