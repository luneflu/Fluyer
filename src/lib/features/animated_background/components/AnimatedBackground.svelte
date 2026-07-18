<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
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
			const prominentColors = (await prominent(image, {
				amount: 10
			})) as number[][];
			colors = prominentColors.map((color) => [color[0], color[1], color[2]] as RGB);
		} else {
			let paletteColors = (await ColorThief.getPalette(image, {
				colorCount: 10
			}))!;
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
		console.log(balancedColors);

		return balancedColors.map((color) => ({
			r: color[0],
			g: color[1],
			b: color[2]
		}));
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
			const dpr = monitor?.scaleFactor ?? 1;

			currentWidth = (monitor?.size.width ?? 0) / dpr;
			currentHeight = (monitor?.size.height ?? 0) / dpr;
		}

		if (currentCoverArt !== null && !MetadataService.isDefaultCoverArt(currentCoverArt)) {
			URL.revokeObjectURL(currentCoverArt);
			currentCoverArt = null;
		}

		currentCoverArt = newCoverArt;
		currentMusicPath = newMusicPath ?? null;

		await TauriBackgroundAPI.updateBackground(await getColors(), currentWidth, currentHeight);

		lastRenderedWidth = currentWidth;
		lastRenderedHeight = currentHeight;

		if (!isInitialized) {
			isInitialized = true;
			// Note: Why? To prevent updateBackground from being called multiple times
			// Since effects references multiple stores
			setTimeout(() => {
				canUpdate = true;
			}, 1000);

			console.log('AnimatedBackground is initialized (WGPU)');
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
		console.log('Updating background from effect');
		updateBackground();
	});

	$effect(() => {
		settingStore.animatedBackground.trigger;
		console.log('Updating background from trigger');
		updateBackground(true);
	});

	async function restoreBackground() {
		if (!isInitialized) return;
		await TauriBackgroundAPI.restoreBackground();
	}

	onMount(async () => {
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
	});
</script>

<svelte:window onresize={onWindowResize} />
