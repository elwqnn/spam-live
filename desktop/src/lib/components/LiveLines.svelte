<script lang="ts" module>
	import { FIELD, program } from '#lib/shader.js';

	const SCALE = 360;
	const LEVELS = 9;
	const FRAGMENT = `#extension GL_OES_standard_derivatives : enable
${FIELD}uniform vec2 o;
uniform float d;
uniform float t;
void main(){
  float f;
  vec3 col=field((o+gl_FragCoord.xy/d)/${SCALE}.,t,f);
  float v=f*${LEVELS}.;
  float a=1.-min(abs(fract(v-.5)-.5)/max(fwidth(v),1e-4)/d,1.);
  gl_FragColor=vec4(col*a,a);
}`;

	type Shared = {
		gl: WebGLRenderingContext;
		o: WebGLUniformLocation | null;
		d: WebGLUniformLocation | null;
		t: WebGLUniformLocation | null;
		still: boolean;
		start: number;
		resize: ResizeObserver;
		visible: IntersectionObserver;
	};

	const shown: HTMLCanvasElement[] = [];
	let count = 0;
	let shared: Shared | null = null;
	let broken = false;
	let frame = 0;

	function hide(canvas: HTMLCanvasElement) {
		const index = shown.indexOf(canvas);
		if (index >= 0) shown.splice(index, 1);
	}

	function render(s: Shared, canvas: HTMLCanvasElement) {
		const ctx = canvas.getContext('2d');
		const ratio = Math.min(devicePixelRatio, 2);
		const width = Math.round(canvas.clientWidth * ratio);
		const height = Math.round(canvas.clientHeight * ratio);
		if (!ctx || !width || !height) return;
		const source = s.gl.canvas as HTMLCanvasElement;
		if (canvas.width !== width) canvas.width = width;
		if (canvas.height !== height) canvas.height = height;
		if (source.width < width) source.width = width;
		if (source.height < height) source.height = height;
		const rect = canvas.getBoundingClientRect();
		s.gl.viewport(0, 0, width, height);
		s.gl.uniform2f(s.o, rect.left + scrollX, -(rect.top + scrollY + height / ratio));
		s.gl.uniform1f(s.d, ratio);
		s.gl.uniform1f(s.t, 20 + (s.still ? 0 : (performance.now() - s.start) / 1000));
		s.gl.drawArrays(s.gl.TRIANGLES, 0, 3);
		ctx.clearRect(0, 0, width, height);
		ctx.drawImage(source, 0, source.height - height, width, height, 0, 0, width, height);
	}

	function loop() {
		frame = 0;
		if (!shared || !shown.length) return;
		for (const canvas of shown) render(shared, canvas);
		frame = requestAnimationFrame(loop);
	}

	function init(): Shared | null {
		const gl = document.createElement('canvas').getContext('webgl', { antialias: false });
		const linked = gl?.getExtension('OES_standard_derivatives') && program(gl, FRAGMENT);
		if (!gl || !linked) {
			gl?.getExtension('WEBGL_lose_context')?.loseContext();
			broken = true;
			return null;
		}
		const s: Shared = {
			gl,
			o: gl.getUniformLocation(linked, 'o'),
			d: gl.getUniformLocation(linked, 'd'),
			t: gl.getUniformLocation(linked, 't'),
			still: matchMedia('(prefers-reduced-motion: reduce)').matches,
			start: performance.now(),
			resize: new ResizeObserver((entries) => {
				for (const entry of entries) render(s, entry.target as HTMLCanvasElement);
			}),
			visible: new IntersectionObserver((entries) => {
				for (const entry of entries) {
					const canvas = entry.target as HTMLCanvasElement;
					hide(canvas);
					if (!entry.isIntersecting) continue;
					shown.push(canvas);
					if (s.still) render(s, canvas);
				}
				if (!s.still && shown.length) frame ||= requestAnimationFrame(loop);
			})
		};
		return s;
	}

	function attach(canvas: HTMLCanvasElement) {
		if (!shared && !broken) shared = init();
		const s = shared;
		if (!s || !canvas.getContext('2d')) return;
		count++;
		s.resize.observe(canvas);
		s.visible.observe(canvas);

		return () => {
			hide(canvas);
			s.resize.unobserve(canvas);
			s.visible.unobserve(canvas);
			if (--count) return;
			cancelAnimationFrame(frame);
			frame = 0;
			s.resize.disconnect();
			s.visible.disconnect();
			s.gl.getExtension('WEBGL_lose_context')?.loseContext();
			shared = null;
		};
	}
</script>

<script lang="ts">
	import { onMount } from 'svelte';

	let canvas: HTMLCanvasElement;

	onMount(() => attach(canvas));
</script>

<canvas
	bind:this={canvas}
	aria-hidden="true"
	class="pointer-events-none absolute inset-0 -z-10 size-full opacity-30 transition-opacity duration-300 [:hover>&]:opacity-35"
></canvas>
