<script lang="ts">
	import { onDestroy, onMount, tick } from 'svelte';
	import { PageRoutes } from '$lib/constants/PageRoutes';
	import { isAndroid, isLinux } from '$lib/platform';
	import { afterNavigate } from '$app/navigation';
	import MetadataService from '$lib/services/MetadataService.svelte';
	import musicStore from '$lib/stores/music.svelte';
	import LibraryService from '$lib/services/LibraryService.svelte';
	import settingStore from '$lib/stores/setting.svelte';
	import { SettingAnimatedBackgroundType } from '$lib/features/settings/animated_background/types';
	// @ts-ignore
	import * as ColorThief from 'colorthief';
	import { prominent } from 'color.js';
	import TauriBackgroundAPI from '$lib/tauri/TauriBackgroundAPI';
	import { listen } from '@tauri-apps/api/event';
	import type { Unsubscriber } from 'svelte/store';
	import ColorConvert, { type RGB } from 'color-convert';
	import { currentMonitor } from '@tauri-apps/api/window';

	interface Color {
		r: number;
		g: number;
		b: number;
	}

	let isInitialized = $state(false);
	let canUpdate = true;
	let currentCoverArt: string | null = null;
	let currentMusicPath: string | null = null;

	let lastRenderedWidth = 0;
	let lastRenderedHeight = 0;

	let unlistenFocus: Unsubscriber;
	let unlistenTransition: Unsubscriber;
	let libraryInitialized = false;

	let glCanvas: HTMLCanvasElement;
	let gl: WebGLRenderingContext;
	let glProgram: WebGLProgram;
	let glCurrentTex: WebGLTexture | null = null;
	let glNextTex: WebGLTexture | null = null;
	let glMixLoc: WebGLUniformLocation | null = null;
	let glCurrentTexLoc: WebGLUniformLocation | null = null;
	let glNextTexLoc: WebGLUniformLocation | null = null;

	let isCef = $state(false);
	let glReady = false;
	let mountComplete = false;

	const VS_SRC = `
		attribute vec2 a_position;
		varying vec2 v_uv;
		void main() {
			// flip Y so texture top = screen top
			v_uv = vec2(a_position.x * 0.5 + 0.5, 0.5 - a_position.y * 0.5);
			gl_Position = vec4(a_position, 0.0, 1.0);
		}
	`;

	const FS_SRC = `
		precision mediump float;
		varying vec2 v_uv;
		uniform sampler2D u_current;
		uniform sampler2D u_next;
		uniform float u_mix;
		void main() {
			vec4 c1 = texture2D(u_current, v_uv);
			vec4 c2 = texture2D(u_next,    v_uv);
			gl_FragColor = mix(c1, c2, u_mix);
		}
	`;

	function compileShader(type: number, src: string): WebGLShader {
		const shader = gl.createShader(type)!;
		gl.shaderSource(shader, src);
		gl.compileShader(shader);
		if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
			throw new Error('Shader compile error: ' + gl.getShaderInfoLog(shader));
		}
		return shader;
	}

	function initWebGL(): boolean {
		const ctx = glCanvas.getContext('webgl') ?? glCanvas.getContext('experimental-webgl');
		if (!ctx) {
			console.error('WebGL not available');
			return false;
		}
		gl = ctx as WebGLRenderingContext;

		const vs = compileShader(gl.VERTEX_SHADER, VS_SRC);
		const fs = compileShader(gl.FRAGMENT_SHADER, FS_SRC);

		glProgram = gl.createProgram()!;
		gl.attachShader(glProgram, vs);
		gl.attachShader(glProgram, fs);
		gl.linkProgram(glProgram);
		if (!gl.getProgramParameter(glProgram, gl.LINK_STATUS)) {
			throw new Error('Program link error: ' + gl.getProgramInfoLog(glProgram));
		}
		gl.useProgram(glProgram);

		// Full-screen quad (two triangles)
		const buf = gl.createBuffer();
		gl.bindBuffer(gl.ARRAY_BUFFER, buf);
		gl.bufferData(
			gl.ARRAY_BUFFER,
			new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
			gl.STATIC_DRAW
		);
		const posLoc = gl.getAttribLocation(glProgram, 'a_position');
		gl.enableVertexAttribArray(posLoc);
		gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 0, 0);

		glMixLoc = gl.getUniformLocation(glProgram, 'u_mix');
		glCurrentTexLoc = gl.getUniformLocation(glProgram, 'u_current');
		glNextTexLoc = gl.getUniformLocation(glProgram, 'u_next');

		glReady = true;
		return true;
	}

	function createGLTexture(width: number, height: number, data: number[] | Uint8Array): WebGLTexture {
		const tex = gl.createTexture()!;
		gl.bindTexture(gl.TEXTURE_2D, tex);
		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
		gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
		const pixels = data instanceof Uint8Array ? data : new Uint8Array(data);
		gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, width, height, 0, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
		return tex;
	}

	let animationFrameId: number;
	let transitionStart: number | null = null;
	const TRANSITION_DURATION = 750; // ms

	function drawGL(timestamp: number) {
		if (!gl || !glProgram || !glCurrentTex || !glNextTex) return;

		if (transitionStart === null) transitionStart = timestamp;
		const mix = Math.min((timestamp - transitionStart) / TRANSITION_DURATION, 1.0);

		gl.viewport(0, 0, glCanvas.width, glCanvas.height);
		gl.clearColor(0, 0, 0, 1);
		gl.clear(gl.COLOR_BUFFER_BIT);

		gl.useProgram(glProgram);
		gl.uniform1f(glMixLoc, mix);

		gl.activeTexture(gl.TEXTURE0);
		gl.bindTexture(gl.TEXTURE_2D, glCurrentTex);
		gl.uniform1i(glCurrentTexLoc, 0);

		gl.activeTexture(gl.TEXTURE1);
		gl.bindTexture(gl.TEXTURE_2D, glNextTex);
		gl.uniform1i(glNextTexLoc, 1);

		gl.drawArrays(gl.TRIANGLES, 0, 6);

		if (mix >= 1.0) {
			// Promote next → current; free old current
			transitionStart = null;
			if (glCurrentTex) gl.deleteTexture(glCurrentTex);
			glCurrentTex = glNextTex;
			glNextTex = null;

			if (!libraryInitialized) {
				libraryInitialized = true;
				LibraryService.initialize();
			}
		} else {
			animationFrameId = requestAnimationFrame(drawGL);
		}
	}

	function renderStill() {
		// Draw the current texture at mix=1 with itself so the screen stays filled
		if (!gl || !glCurrentTex) return;
		gl.viewport(0, 0, glCanvas.width, glCanvas.height);
		gl.clear(gl.COLOR_BUFFER_BIT);
		gl.useProgram(glProgram);
		gl.uniform1f(glMixLoc, 0.0);
		gl.activeTexture(gl.TEXTURE0);
		gl.bindTexture(gl.TEXTURE_2D, glCurrentTex);
		gl.uniform1i(glCurrentTexLoc, 0);
		gl.activeTexture(gl.TEXTURE1);
		gl.bindTexture(gl.TEXTURE_2D, glCurrentTex);
		gl.uniform1i(glNextTexLoc, 1);
		gl.drawArrays(gl.TRIANGLES, 0, 6);
	}

	function triggerTransition() {
		if (animationFrameId) cancelAnimationFrame(animationFrameId);
		transitionStart = null;
		animationFrameId = requestAnimationFrame(drawGL);
	}

	async function getColors(): Promise<Color[] | null> {
		if (!currentCoverArt) return null;
		let image = new Image();
		image.crossOrigin = 'anonymous';
		image.src = currentCoverArt;

		if (!image.complete) {
			await new Promise((resolve, reject) => {
				image.onload = () => resolve(null);
				image.onerror = (err) => reject(err);
			});
		}

		let colors: RGB[] = [];
		if (settingStore.animatedBackground.type === SettingAnimatedBackgroundType.Prominent) {
			// @ts-ignore
			const prominentColors = (await prominent(image, { amount: 10 })) as number[][];
			colors = prominentColors.map((color) => [color[0], color[1], color[2]] as RGB);
		} else {
			let paletteColors = (await ColorThief.getPalette(image, { colorCount: 10 }))!;
			colors = paletteColors.map((color: ColorThief.Color) => {
				let rgbColor = color.rgb();
				return [rgbColor.r, rgbColor.g, rgbColor.b] as RGB;
			});
		}

		let balancedColors: RGB[] = colors.map((color) => {
			let [h, s, l] = ColorConvert.rgb.hsl(color[0], color[1], color[2]);
			if (l > 60) l = 60;
			if (MetadataService.isDefaultCoverArt(currentCoverArt)) {
				l = 60;
				s = 60;
			}
			return ColorConvert.hsl.rgb(h, s, l);
		});

		return balancedColors.map((color) => ({ r: color[0], g: color[1], b: color[2] }));
	}

	async function updateBackground(force = false) {
		if (!canUpdate) return;
		if (!isInitialized) {
			console.log('AnimatedBackground is initializing...');
			canUpdate = false;
		}

		const newMusicPath = musicStore.currentMusic?.path;
		if (currentMusicPath === newMusicPath && !force) return;

		const newCoverArt = await MetadataService.getMusicCoverArt(musicStore.currentMusic);

		let currentWidth = window.innerWidth;
		let currentHeight = window.innerHeight;

		if (!isInitialized) {
			const monitor = await currentMonitor();
			const dpr = window.devicePixelRatio;
			currentWidth = (monitor?.size.width ?? 0) / dpr;
			currentHeight = (monitor?.size.height ?? 0) / dpr;
		}

		if (currentCoverArt !== null && !MetadataService.isDefaultCoverArt(currentCoverArt)) {
			URL.revokeObjectURL(currentCoverArt);
			currentCoverArt = null;
		}

		currentCoverArt = newCoverArt;
		currentMusicPath = newMusicPath ?? null;

		const result = await TauriBackgroundAPI.updateBackground(
			await getColors(),
			currentWidth,
			currentHeight
		);

		// CEF path: render via WebGL canvas
		if (isCef && glReady && result) {
			const [data, texWidth, texHeight] = result;

			if (!glCurrentTex) {
				// First frame — show immediately, no transition
				glCurrentTex = createGLTexture(texWidth, texHeight, data);
				renderStill();
				if (!libraryInitialized) {
					libraryInitialized = true;
					LibraryService.initialize();
				}
			} else {
				// Swap in new texture as "next" and crossfade
				if (glNextTex) gl.deleteTexture(glNextTex);
				glNextTex = createGLTexture(texWidth, texHeight, data);
				triggerTransition();
			}
		}

		lastRenderedWidth = currentWidth;
		lastRenderedHeight = currentHeight;

		if (!isInitialized) {
			isInitialized = true;
			setTimeout(() => { canUpdate = true; }, 1000);
			console.log('AnimatedBackground is initialized (WebGL/Native)');
		}
	}

	function onWindowResize() {
		if (lastRenderedWidth === 0 || lastRenderedHeight === 0) {
			lastRenderedWidth = window.innerWidth;
			lastRenderedHeight = window.innerHeight;
			return;
		}

		const widthDiff = Math.abs(window.innerWidth - lastRenderedWidth) / lastRenderedWidth;
		const heightDiff = Math.abs(window.innerHeight - lastRenderedHeight) / lastRenderedHeight;

		if (widthDiff >= 0.25 || heightDiff >= 0.25) {
			console.log('Resized by 25%, updating background');
			if (isCef && glReady && glCanvas) {
				glCanvas.width = window.innerWidth;
				glCanvas.height = window.innerHeight;
				renderStill();
			}
			updateBackground(true);
		}
	}

	if (isLinux())
		afterNavigate((navigation) => {
			if (navigation.from?.route.id !== PageRoutes.VISUALIZER) return;
			updateBackground(true);
		});

	$effect(() => {
		musicStore.currentMusic;
		if (!mountComplete) return;
		console.log('Updating background from effect');
		updateBackground();
	});

	$effect(() => {
		settingStore.animatedBackground.trigger;
		if (!mountComplete) return;
		console.log('Updating background from trigger');
		updateBackground(true);
	});

	async function restoreBackground() {
		if (!isInitialized) return;

		if (isCef && glReady && glCurrentTex) {
			// Crossfade back to the cached current texture (copy it into "next")
			if (glNextTex) gl.deleteTexture(glNextTex);
			// Re-upload current texture pixels as next (effectively a no-op fade)
			glNextTex = glCurrentTex;
			glCurrentTex = null; // will be repopulated on next updateBackground
			triggerTransition();
		} else {
			await TauriBackgroundAPI.restoreBackground();
		}
	}

	onMount(async () => {
		isCef = await TauriBackgroundAPI.isCefEnabled();
		if (isCef) {
			// Let Svelte render the {#if isCef} canvas into the DOM first.
			await tick();
			await tick();
			if (glCanvas) {
				glCanvas.width = window.innerWidth;
				glCanvas.height = window.innerHeight;
			}
			initWebGL();
		}

		// Call updateBackground only after WebGL is ready so the guard passes.
		mountComplete = true;
		updateBackground(true);
		if (isAndroid()) unlistenFocus = await listen('tauri://focus', restoreBackground);

		unlistenTransition = await TauriBackgroundAPI.listenTransitionComplete(() => {
			if (!libraryInitialized) {
				libraryInitialized = true;
				LibraryService.initialize();
			}
		});
	});

	onDestroy(() => {
		if (unlistenFocus) unlistenFocus();
		if (unlistenTransition) unlistenTransition();
		if (animationFrameId) cancelAnimationFrame(animationFrameId);
		if (gl) {
			if (glCurrentTex) gl.deleteTexture(glCurrentTex);
			if (glNextTex) gl.deleteTexture(glNextTex);
		}
	});
</script>

<svelte:window onresize={onWindowResize} />

{#if isCef}
	<canvas
		bind:this={glCanvas}
		style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: -1; pointer-events: none;"
	></canvas>
{/if}
