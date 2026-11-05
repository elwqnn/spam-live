<script lang="ts">
	import type { Attachment } from 'svelte/attachments';
	import { FIELD, program } from '#lib/shader.js';

	const FRAGMENT = `${FIELD}uniform vec2 r;
uniform float t;
void main(){
  vec2 uv=gl_FragCoord.xy/r;
  float f;
  gl_FragColor=vec4(field(vec2(uv.x*r.x/r.y,uv.y)*1.4,t,f),1.);
}`;

	const shader: Attachment<HTMLCanvasElement> = (canvas) => {
		const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
		const gl = canvas.getContext('webgl', { antialias: false, preserveDrawingBuffer: still });
		if (!gl) return;
		const lose = () => gl.getExtension('WEBGL_lose_context')?.loseContext();
		const linked = program(gl, FRAGMENT);
		if (!linked) {
			lose();
			return;
		}
		const size = gl.getUniformLocation(linked, 'r');
		const time = gl.getUniformLocation(linked, 't');

		const start = performance.now();
		let frame = 0;

		const draw = () => {
			gl.viewport(0, 0, canvas.width, canvas.height);
			gl.uniform2f(size, canvas.width, canvas.height);
			gl.uniform1f(time, 20 + (still ? 0 : (performance.now() - start) / 1000));
			gl.drawArrays(gl.TRIANGLES, 0, 3);
		};
		const loop = () => {
			draw();
			frame = requestAnimationFrame(loop);
		};
		const stop = () => {
			cancelAnimationFrame(frame);
			frame = 0;
		};

		const resize = new ResizeObserver(() => {
			canvas.width = Math.max(1, canvas.clientWidth);
			canvas.height = Math.max(1, canvas.clientHeight);
			draw();
		});
		const view = new IntersectionObserver(([entry]) => {
			stop();
			if (entry.isIntersecting && !still) loop();
		});
		resize.observe(canvas);
		view.observe(canvas);

		return () => {
			stop();
			resize.disconnect();
			view.disconnect();
			lose();
		};
	};
</script>

<div class="fallback absolute inset-0" aria-hidden="true">
	<canvas class="block size-full" {@attach shader}></canvas>
	<div class="grain absolute inset-0 opacity-8 mix-blend-overlay"></div>
	<div class="shade absolute inset-0"></div>
</div>

<style>
	.fallback {
		background: linear-gradient(120deg, var(--plum), var(--terra) 55%, var(--straw));
	}
	.shade {
		background:
			linear-gradient(
				180deg,
				rgba(23, 23, 21, 0.58),
				rgba(23, 23, 21, 0.4) 7rem,
				transparent 16rem
			),
			radial-gradient(
				85% 55% at 50% 48%,
				rgba(23, 23, 21, 0.52),
				rgba(23, 23, 21, 0.47) 50%,
				transparent 92%
			);
	}
</style>
