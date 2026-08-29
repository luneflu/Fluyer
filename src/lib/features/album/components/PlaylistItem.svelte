<script lang="ts">
	import type { PlaylistData } from '$lib/features/music/types';
	import playlistStore from '$lib/stores/playlist.svelte';
	import { usePlaylistItem } from '../viewmodels/usePlaylistItem.svelte';

	interface Props {
		playlist: PlaylistData;
		visible?: boolean;
	}

	let { playlist, visible = false }: Props = $props();

	const vm = usePlaylistItem(
		() => playlist,
		() => visible
	);
</script>

<div class="col-auto row-[1] h-fit px-3 pb-3">
	<div class="relative w-full">
		{#if playlistStore.selectedPlaylist?.id === playlist.id}
			<div
				class="absolute left-0 top-0 z-10 h-full w-full
            rounded border-2 border-white"
			></div>
		{:else}
			<div
				class="playlist-item-actions absolute z-20 h-full w-full cursor-pointer
                rounded border-2 border-white bg-white/20"
				onclick={vm.selectPlaylist}
			></div>
		{/if}
		{#await vm.coverArt}
			<div class="aspect-square w-full"></div>
		{:then image}
			{#if image}
				<img
					class="anim anim-fade-in aspect-square w-full rounded object-cover"
					src={image}
					alt="Playlist"
				/>
			{:else}
				<div class="rounded aspect-square w-full"></div>
			{/if}
		{/await}
	</div>
	<p
		class="animate-scroll-overflow-text mt-2 overflow-hidden whitespace-nowrap font-medium md:text-lg"
	>
		{playlist.title || playlist.name}
	</p>
	<p
		class="text-opacity-background-80 animate-scroll-overflow-text overflow-hidden whitespace-nowrap text-[15px] md:text-base"
	>
		{playlist.artist || `${playlist.paths.length} Tracks`}
	</p>
</div>

<style lang="scss">
	.playlist-item-actions {
		opacity: 0;
		transition: opacity 0.75s;

		&:hover {
			opacity: 1;
			transition: opacity 0.5s;
		}
	}
</style>
