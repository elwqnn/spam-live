<script lang="ts">
	import { onMount } from 'svelte';
	import { FIELD, program } from '#lib/shader.js';

	const FRAGMENT = `${FIELD}uniform vec2 r;
uniform float t;
void main(){
  vec2 uv=gl_FragCoord.xy/r;
  float f;
  gl_FragColor=vec4(field(vec2(uv.x*r.x/r.y,uv.y)*1.4,t,f),1.);
}`;

	let canvas: HTMLCanvasElement;
	let failed = $state(false);

	onMount(() => {
		const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
		const gl = canvas.getContext('webgl', { antialias: false, preserveDrawingBuffer: still });
		const linked = gl && program(gl, FRAGMENT);
		if (!gl || !linked) {
			failed = true;
			return;
		}
		const r = gl.getUniformLocation(linked, 'r');
		const t = gl.getUniformLocation(linked, 't');

		const start = performance.now();
		let frame = 0;

		const render = () => {
			gl.viewport(0, 0, canvas.width, canvas.height);
			gl.uniform2f(r, canvas.width, canvas.height);
			gl.uniform1f(t, 20 + (still ? 0 : (performance.now() - start) / 1000));
			gl.drawArrays(gl.TRIANGLES, 0, 3);
		};
		const loop = () => {
			render();
			frame = requestAnimationFrame(loop);
		};
		const stop = () => {
			cancelAnimationFrame(frame);
			frame = 0;
		};

		const resize = new ResizeObserver(() => {
			const ratio = Math.min(devicePixelRatio, 1);
			canvas.width = Math.max(1, Math.round(canvas.clientWidth * ratio));
			canvas.height = Math.max(1, Math.round(canvas.clientHeight * ratio));
			render();
		});
		const visible = new IntersectionObserver(([entry]) => {
			if (still) return;
			if (entry.isIntersecting) frame ||= requestAnimationFrame(loop);
			else stop();
		});
		resize.observe(canvas);
		visible.observe(canvas);

		return () => {
			stop();
			resize.disconnect();
			visible.disconnect();
			gl.getExtension('WEBGL_lose_context')?.loseContext();
		};
	});
</script>

<div class={['absolute inset-0', failed && 'shader-fallback']} aria-hidden="true">
	<canvas bind:this={canvas} class="block size-full"></canvas>
</div>
