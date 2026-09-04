<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { PageRoutes } from '$lib/constants/PageRoutes';
	import { isAndroid, isLinux, isWindows } from '$lib/platform';
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
	let libraryInitialized = false;

	// Canvas 2D state
	let canvas: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null = null;
	let currentBitmap: ImageBitmap | null = null;
	let nextBitmap: ImageBitmap | null = null;
	let mountComplete = false;

	let animationFrameId: number;
	let transitionStart: number | null = null;
	const TRANSITION_DURATION = 750; // ms

	function initCanvas2D() {
		ctx = canvas.getContext('2d');
	}

	function renderStill() {
		if (!ctx || !currentBitmap) return;
		ctx.clearRect(0, 0, canvas.width, canvas.height);
		ctx.globalAlpha = 1.0;
		ctx.drawImage(currentBitmap, 0, 0, canvas.width, canvas.height);
	}

	function drawFadeIn(timestamp: number) {
		if (!ctx || !currentBitmap) return;

		if (transitionStart === null) transitionStart = timestamp;
		const alpha = Math.min((timestamp - transitionStart) / TRANSITION_DURATION, 1.0);

		ctx.clearRect(0, 0, canvas.width, canvas.height);
		ctx.globalAlpha = alpha;
		ctx.drawImage(currentBitmap, 0, 0, canvas.width, canvas.height);
		ctx.globalAlpha = 1.0;

		if (alpha >= 1.0) {
			transitionStart = null;
			if (!libraryInitialized) {
				libraryInitialized = true;
				LibraryService.initialize();
			}
		} else {
			animationFrameId = requestAnimationFrame(drawFadeIn);
		}
	}

	function drawFrame(timestamp: number) {
		if (!ctx || !currentBitmap || !nextBitmap) return;

		if (transitionStart === null) transitionStart = timestamp;
		const mix = Math.min((timestamp - transitionStart) / TRANSITION_DURATION, 1.0);

		ctx.globalAlpha = 1.0;
		ctx.drawImage(currentBitmap, 0, 0, canvas.width, canvas.height);
		ctx.globalAlpha = mix;
		ctx.drawImage(nextBitmap, 0, 0, canvas.width, canvas.height);
		ctx.globalAlpha = 1.0;

		if (mix >= 1.0) {
			transitionStart = null;
			currentBitmap.close();
			currentBitmap = nextBitmap;
			nextBitmap = null;

			if (!libraryInitialized) {
				libraryInitialized = true;
				LibraryService.initialize();
			}
		} else {
			animationFrameId = requestAnimationFrame(drawFrame);
		}
	}

	function triggerTransition() {
		if (animationFrameId) cancelAnimationFrame(animationFrameId);
		transitionStart = null;
		animationFrameId = requestAnimationFrame(drawFrame);
	}

	function triggerFadeIn() {
		if (animationFrameId) cancelAnimationFrame(animationFrameId);
		transitionStart = null;
		animationFrameId = requestAnimationFrame(drawFadeIn);
	}

	async function bitmapFromRgba(
		data: number[] | Uint8Array,
		w: number,
		h: number
	): Promise<ImageBitmap> {
		const pixels = data instanceof Uint8Array ? data : new Uint8Array(data);
		const imageData = new ImageData(
			new Uint8ClampedArray(pixels.buffer, pixels.byteOffset, pixels.byteLength),
			w,
			h
		);
		return createImageBitmap(imageData);
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
			if (MetadataService.isDefaultCoverArt(currentCoverArt)) {
				l = 50;
				while (s > 40) s *= 0.9;
			} else {
				while (l > 45) l *= 0.9;
				while (s > 50) s *= 0.9;
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
			currentWidth = Math.floor((monitor?.size.width ?? window.innerWidth ?? 0) / dpr);
			currentHeight = Math.floor((monitor?.size.height ?? window.innerHeight ?? 0) / dpr);
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

		if (result && ctx) {
			const [data, texWidth, texHeight] = result;

			if (!currentBitmap) {
				// First frame — fade in from transparent
				currentBitmap = await bitmapFromRgba(data, texWidth, texHeight);
				triggerFadeIn();
			} else {
				// Interrupted transition: capture current canvas state as starting bitmap
				if (animationFrameId) cancelAnimationFrame(animationFrameId);
				if (nextBitmap || transitionStart !== null) {
					const snapshot = await createImageBitmap(canvas);
					currentBitmap.close();
					if (nextBitmap) nextBitmap.close();
					currentBitmap = snapshot;
					nextBitmap = null;
					transitionStart = null;
				}
				nextBitmap = await bitmapFromRgba(data, texWidth, texHeight);
				triggerTransition();
			}
		}

		lastRenderedWidth = currentWidth;
		lastRenderedHeight = currentHeight;

		if (!isInitialized) {
			isInitialized = true;
			setTimeout(() => {
				canUpdate = true;
			}, 1000);
			console.log('AnimatedBackground is initialized (Canvas 2D)');
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
			canvas.width = window.innerWidth;
			canvas.height = window.innerHeight;
			renderStill();
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
		if (!isInitialized || !ctx || !currentBitmap) return;
		// Re-draw current bitmap — no Rust call needed
		renderStill();
	}

	onMount(async () => {
		canvas.width = window.innerWidth;
		canvas.height = window.innerHeight;
		initCanvas2D();

		mountComplete = true;
		updateBackground(true);
		if (isAndroid()) unlistenFocus = await listen('tauri://focus', restoreBackground);
	});

	onDestroy(() => {
		if (unlistenFocus) unlistenFocus();
		if (animationFrameId) cancelAnimationFrame(animationFrameId);
		if (currentBitmap) currentBitmap.close();
		if (nextBitmap) nextBitmap.close();
	});
</script>

<svelte:window onresize={onWindowResize} />

<canvas
	bind:this={canvas}
	style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: -1; pointer-events: none;"
	class:rounded-lg={isLinux() || isWindows()}
></canvas>
