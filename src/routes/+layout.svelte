<script lang="ts">
	import 'animate.css';
	import AnimatedBackground from '$lib/features/animated_background/components/AnimatedBackground.svelte';
	import '../app.scss';
	import { isDesktop, isLinux } from '$lib/platform';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { onMount } from 'svelte';
	import { PageRoutes } from '$lib/constants/PageRoutes';
	import { page } from '$app/state';
	import ToastService from '$lib/services/ToastService.svelte';
	import PersistentStoreService from '$lib/services/PersistentStoreService.svelte';
	import musicStore from '$lib/stores/music.svelte';
	import MusicPlayerService from '$lib/services/MusicPlayerService.svelte';
	import UIInteractionService from '$lib/services/UIInteractionService.svelte';
	import MobileService from '$lib/services/MobileService.svelte';
	import FolderService from '$lib/services/FolderService.svelte';
	import LogService from '$lib/services/LogService.svelte';
	import Font from '$lib/ui/font/Font.svelte';
	import FilterBar from '$lib/features/filterbar/components/FilterBar.svelte';
	import TitleBar from '$lib/features/titlebar/components/TitleBar.svelte';
	import MetadataService from '$lib/services/MetadataService.svelte';
	import ToastContainer from '$lib/ui/toast/ToastContainer.svelte';
	import CreatePlaylistModal from '$lib/features/playlist/components/CreatePlaylistModal.svelte';
	import PlaylistService from '$lib/services/PlaylistService.svelte';
	import UpdateService from '$lib/services/UpdateService.svelte';
	import TauriBackgroundAPI from '$lib/tauri/TauriBackgroundAPI';
	import appStore from '$lib/stores/app.svelte';

	if (isLinux()) {
		import('$lib/scss/linux.scss');
	}

	interface Props {
		children?: import('svelte').Snippet;
	}

	let { children }: Props = $props();
	let isAppReady = $state(false);

	onMount(async () => {
		const now = performance.now();
		await Promise.all([
			LogService.initialize(),
			ToastService.initialize(),
			PersistentStoreService.initialize(),
			MusicPlayerService.initialize(),
			UIInteractionService.initialize(),
			MobileService.initialize(),
			FolderService.initialize(),
			MetadataService.initialize(),
			PlaylistService.initialize(),
			(async () => {
				appStore.isCefEnabled = await TauriBackgroundAPI.isCefEnabled();
				console.log(appStore.isCefEnabled);
			})()
		]);

		if (isDesktop()) {
			await getCurrentWindow().show();
			if (!(await getCurrentWindow().isMaximized())) await getCurrentWindow().toggleMaximize();
		}

		isAppReady = true;

		console.log(`Front-end is initialized. Took ${performance.now() - now} ms`);

		if (!appStore.isCefEnabled) {
			UpdateService.checkForUpdates();
		}
	});
</script>

<Font />
<ToastContainer />
{#if isAppReady}
	<AnimatedBackground />
{/if}
<div class="scrollbar-hidden fixed h-screen w-screen">
	{@render children?.()}
</div>
{#if isDesktop() && page.url.pathname !== PageRoutes.PLAY && !appStore.isCefEnabled}
	<TitleBar />
{/if}
{#if musicStore.isLibraryLoaded}
	{#if [PageRoutes.HOME, PageRoutes.HOME_PRODUCTION].includes(page.url.pathname)}
		<FilterBar />
	{/if}
{/if}
<CreatePlaylistModal />
