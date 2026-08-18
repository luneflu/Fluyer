<script lang="ts">
	import { isMobile } from '$lib/platform';
	import mobileStore from '$lib/stores/mobile.svelte';
	import filterBarStore from '$lib/stores/filterBar.svelte';
	import musicStore from '$lib/stores/music.svelte';
	import { MusicListType } from '$lib/features/music/types';
	import Intro from '$lib/features/intro/components/Intro.svelte';
	import AlbumList from '$lib/features/album/components/AlbumList.svelte';
	import MusicQueueList from '$lib/features/music_queue/components/MusicQueueList.svelte';
	import CollectionInfo from '$lib/features/collection/components/CollectionInfo.svelte';
	import MusicList from '$lib/features/music/components/MusicList.svelte';
	import Menu from '$lib/features/menu/components/Menu.svelte';
	import PlayerBar from '$lib/features/playerbar/components/PlayerBar.svelte';
	import modalStore from '$lib/stores/modal.svelte';

	let paddingTop = $derived((isMobile() ? mobileStore.statusBarHeight : 0) + filterBarStore.height);

	let gridClass = $derived.by(() => {
		switch (musicStore.listType) {
			case MusicListType.All:
				return 'grid-rows-[min-content_min-content_auto_min-content]';
			case MusicListType.Music:
				return 'grid-rows-[auto_min-content]';
			case MusicListType.Folder:
				return 'grid-rows-[min-content_auto_min-content]';
			case MusicListType.Playlist:
				return 'grid-rows-[min-content_min-content_auto_min-content]';
			default:
				return 'grid-rows-[auto_min-content]';
		}
	});

	let tooltipVisible = $state(false);
</script>

{#if musicStore.isLibraryLoaded === false}
	<Intro />
{:else if musicStore.isLibraryLoaded === true}
	<!--{#if isDesktop()}-->
	<!--    <Equalizer />-->
	<!--{/if}-->
	<MusicQueueList />
	<Menu />
	<div
		class="grid h-full w-full {gridClass}
	 			{modalStore.show ? 'opacity-10 blur-sm' : ''} transition-opacity duration-300"
		style="padding-top: {paddingTop}px;"
	>
		{#if [MusicListType.All, MusicListType.Album, MusicListType.Playlist].includes(musicStore.listType)}
			<AlbumList />
		{/if}
		{#if [MusicListType.All, MusicListType.Folder, MusicListType.Playlist].includes(musicStore.listType)}
			<CollectionInfo />
		{/if}
		{#if [MusicListType.All, MusicListType.Music, MusicListType.Folder, MusicListType.Playlist].includes(musicStore.listType)}
			<MusicList {tooltipVisible} />
		{/if}
		<PlayerBar bind:tooltipVisible />
	</div>
{/if}
