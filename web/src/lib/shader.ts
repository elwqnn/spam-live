const VERTEX = 'attribute vec2 a;void main(){gl_Position=vec4(a,0.,1.);}';

export const FIELD = `precision highp float;
uniform vec3 plum;
uniform vec3 terra;
uniform vec3 straw;
uniform vec3 slate;
uniform vec3 moss;
float h(vec2 p){return fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453);}
float n(vec2 p){vec2 i=floor(p);vec2 f=fract(p);vec2 u=f*f*(3.-2.*f);
  return mix(mix(h(i),h(i+vec2(1.,0.)),u.x),mix(h(i+vec2(0.,1.)),h(i+vec2(1.,1.)),u.x),u.y);}
float fbm(vec2 p){float v=0.;float a=.5;for(int k=0;k<5;k++){v+=a*n(p);p=p*2.03+vec2(1.7,9.2);a*=.5;}return v;}
vec3 field(vec2 p,float t,out float f){
  float T=t*.035;
  vec2 q=vec2(fbm(p+vec2(0.,T)),fbm(p+vec2(5.2,1.3)-T));
  vec2 w=vec2(fbm(p+2.8*q+vec2(1.7,9.2)+T*1.3),fbm(p+2.8*q+vec2(8.3,2.8)-T));
  f=fbm(p+3.2*w);
  vec3 col=mix(plum,terra,smoothstep(.25,.75,f));
  col=mix(col,straw,smoothstep(.55,.95,w.y)*.75);
  col=mix(col,slate,smoothstep(.45,.9,q.x)*.55);
  col=mix(col,moss,smoothstep(.6,.95,q.y*w.x*1.6)*.35);
  return col*(.78+.3*f);
}
`;

const PALETTE = ['plum', 'terra', 'straw', 'slate', 'moss'];

export function program(gl: WebGLRenderingContext, fragment: string) {
	const program = gl.createProgram();
	for (const [type, source] of [
		[gl.VERTEX_SHADER, VERTEX],
		[gl.FRAGMENT_SHADER, fragment]
	] as const) {
		const part = gl.createShader(type);
		if (!part) return null;
		gl.shaderSource(part, source);
		gl.compileShader(part);
		gl.attachShader(program, part);
	}
	gl.linkProgram(program);
	if (!gl.getProgramParameter(program, gl.LINK_STATUS)) return null;
	gl.useProgram(program);
	gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
	gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
	const position = gl.getAttribLocation(program, 'a');
	gl.enableVertexAttribArray(position);
	gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0);
	const style = getComputedStyle(document.documentElement);
	for (const name of PALETTE) {
		const hex = style.getPropertyValue(`--${name}`).trim();
		gl.uniform3fv(
			gl.getUniformLocation(program, name),
			[1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
		);
	}
	return program;
}
