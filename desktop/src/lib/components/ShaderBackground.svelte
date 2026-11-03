<script lang="ts">
	import { onMount } from 'svelte';

	const VERTEX = 'attribute vec2 a;void main(){gl_Position=vec4(a,0.,1.);}';
	const FRAGMENT = `precision highp float;
uniform vec2 r;
uniform float t;
float h(vec2 p){return fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453);}
float n(vec2 p){vec2 i=floor(p);vec2 f=fract(p);vec2 u=f*f*(3.-2.*f);
  return mix(mix(h(i),h(i+vec2(1.,0.)),u.x),mix(h(i+vec2(0.,1.)),h(i+vec2(1.,1.)),u.x),u.y);}
float fbm(vec2 p){float v=0.;float a=.5;for(int k=0;k<5;k++){v+=a*n(p);p=p*2.03+vec2(1.7,9.2);a*=.5;}return v;}
void main(){
  vec2 uv=gl_FragCoord.xy/r;
  vec2 p=vec2(uv.x*r.x/r.y,uv.y)*1.4;
  float T=t*.035;
  vec2 q=vec2(fbm(p+vec2(0.,T)),fbm(p+vec2(5.2,1.3)-T));
  vec2 w=vec2(fbm(p+2.8*q+vec2(1.7,9.2)+T*1.3),fbm(p+2.8*q+vec2(8.3,2.8)-T));
  float f=fbm(p+3.2*w);
  vec3 plum=vec3(.33,.23,.47);
  vec3 terra=vec3(.89,.38,.25);
  vec3 straw=vec3(.93,.79,.40);
  vec3 slate=vec3(.27,.52,.70);
  vec3 moss=vec3(.42,.62,.37);
  vec3 col=mix(plum,terra,smoothstep(.25,.75,f));
  col=mix(col,straw,smoothstep(.55,.95,w.y)*.75);
  col=mix(col,slate,smoothstep(.45,.9,q.x)*.55);
  col=mix(col,moss,smoothstep(.6,.95,q.y*w.x*1.6)*.35);
  col*=.78+.3*f;
  gl_FragColor=vec4(col,1.);
}`;

	let canvas: HTMLCanvasElement;
	let failed = $state(false);

	function link(gl: WebGLRenderingContext) {
		const program = gl.createProgram();
		for (const [type, source] of [
			[gl.VERTEX_SHADER, VERTEX],
			[gl.FRAGMENT_SHADER, FRAGMENT]
		] as const) {
			const shader = gl.createShader(type);
			if (!shader) return null;
			gl.shaderSource(shader, source);
			gl.compileShader(shader);
			gl.attachShader(program, shader);
		}
		gl.linkProgram(program);
		return gl.getProgramParameter(program, gl.LINK_STATUS) ? program : null;
	}

	onMount(() => {
		const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
		const gl = canvas.getContext('webgl', { antialias: false, preserveDrawingBuffer: still });
		const program = gl && link(gl);
		if (!gl || !program) {
			failed = true;
			return;
		}
		gl.useProgram(program);
		gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
		gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
		const a = gl.getAttribLocation(program, 'a');
		gl.enableVertexAttribArray(a);
		gl.vertexAttribPointer(a, 2, gl.FLOAT, false, 0, 0);
		const r = gl.getUniformLocation(program, 'r');
		const t = gl.getUniformLocation(program, 't');

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
