export function rateDomain(values: number[]): [number, number] | undefined {
	if (!values.length) return undefined;
	return [
		Math.max(0, Math.floor(Math.min(...values) * 20 - 1)) / 20,
		Math.min(20, Math.ceil(Math.max(...values) * 20 + 1)) / 20
	];
}
