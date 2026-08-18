<script lang="ts">
	import Icon from '$lib/ui/icon/Icon.svelte';
	import { IconType } from '$lib/ui/icon/types';
	import View from '$lib/ui/components/View.svelte';
	import musicStore from '$lib/stores/music.svelte';
	import { MusicListType } from '$lib/features/music/types';
	import folderStore from '$lib/stores/folder.svelte';
	import useCollectionInfo from '$lib/features/collection/viewmodels/useCollectionInfo.svelte';
	import sidebarStore from '$lib/stores/sidebar.svelte';
	import { isLinux, isMobile } from '$lib/platform';
	import playlistStore from '$lib/stores/playlist.svelte';

	const vm = useCollectionInfo();
	let shouldShow = $derived(
		vm.album ||
			folderStore.currentFolder ||
			(musicStore.listType === MusicListType.Playlist && !!playlistStore.selectedPlaylist)
	);
	let isSidebarVisible = $derived(!!sidebarStore.showType);
</script>

{#if shouldShow}
	<View
		class="anim mx-3 mb-2 box-border grid-cols-[auto_max-content] rounded-lg
        px-4 py-2 md:grid
		{isSidebarVisible ? 'anim-fade-out' : 'anim-fade-in'}"
		style="animation-duration: {isLinux() ? '350ms' : '500ms'};"
	>
		<div class="grid items-center">
			<div class="overflow-hidden text-sm font-medium md:text-base">
				<p class="animate-scroll-overflow-text overflow-x-hidden whitespace-nowrap">{vm.label}</p>
			</div>
		</div>
		<div class="w-fit">
			<div></div>
			{#await vm.showBackButton then showBackButton}
				<div
					class="mt-2 grid gap-x-2 md:mt-0 md:gap-x-3"
					class:grid-cols-5={musicStore.listType === MusicListType.Playlist}
					class:grid-cols-4={showBackButton && musicStore.listType !== MusicListType.Playlist}
					class:grid-cols-3={!showBackButton && musicStore.listType !== MusicListType.Playlist}
				>
					{#if showBackButton}
						<button
							class="flex h-6 w-6 items-center justify-center md:h-7 md:w-7"
							onclick={vm.handleBack}
						>
							<Icon type={IconType.AlbumBack} />
						</button>
					{/if}
					<button
						class="flex h-6 w-6 items-center justify-center md:h-7 md:w-7"
						onclick={vm.addMusicListAndPlay}
					>
						<Icon type={IconType.Play} />
					</button>
					<button
						class="flex h-6 w-6 items-center justify-center md:h-7 md:w-7"
						onclick={vm.addMusicList}
					>
						<Icon type={IconType.QueueMusic} />
					</button>
					<button
						class="flex h-6 w-6 items-center justify-center md:h-7 md:w-7"
						onclick={vm.playShuffle}
					>
						<Icon type={IconType.Shuffle} />
					</button>
					{#if musicStore.listType === MusicListType.Playlist}
						<button
							class="flex h-6 w-6 items-center justify-center md:h-7 md:w-7"
							onclick={vm.deletePlaylist}
						>
							<Icon type={IconType.Trash} />
						</button>
					{/if}
				</div>
			{/await}
		</div>
	</View>
{:else}
	<div></div>
{/if}
