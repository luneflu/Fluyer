<script lang="ts">
	import musicStore from '$lib/stores/music.svelte';
	import playlistStore from '$lib/stores/playlist.svelte';
	import sidebarStore from '$lib/stores/sidebar.svelte';
	import { MusicListType } from '$lib/features/music/types';
	import AlbumItem from '$lib/features/album/components/AlbumItem.svelte';
	import PlaylistItem from '$lib/features/album/components/PlaylistItem.svelte';
	import { useAlbumList } from '$lib/features/album/viewmodels/useAlbumList.svelte';

	const vm = useAlbumList();

	let scrollContainer = $state<HTMLDivElement>();

	const isPlaylistMode = $derived(musicStore.listType === MusicListType.Playlist);
	let isHovered = $state(false);
</script>

<svelte:window onresize={vm.updateItemWidth} />

<div
	class="w-screen transition-all duration-300 {sidebarStore.showType
		? 'pointer-events-none opacity-20'
		: ''}"
	style="height: {vm.containerHeight}px;"
	onmouseenter={() => (isHovered = true)}
	onmouseleave={() => (isHovered = false)}
>
	<div
		bind:this={scrollContainer}
		use:vm.scrollable
		class={vm.scrollClass}
		onscroll={vm.handleScroll}
		onwheel={vm.isHorizontal ? (e) => vm.handleWheel(e, scrollContainer) : undefined}
		style="padding-bottom: {vm.bottomPadding}px;"
	>
		{#if isPlaylistMode}
			<!-- Playlist grid layout -->
			<div
				class="grid"
				style="grid-template-columns: repeat({vm.state.columnCount}, minmax(0, 1fr));"
			>
				{#each playlistStore.list as playlist, index}
					{@const hiddenBySidebar = vm.shouldHidePlaylistGridItem(index)}
					{@const inViewport = vm.visibleItems.has(index)}
					<div
						use:vm.observeElement={index}
						class={vm.itemClass(inViewport, hiddenBySidebar)}
						style={vm.itemStyle(hiddenBySidebar)}
						onanimationend={() => vm.handleAnimationEnd(index, hiddenBySidebar)}
					>
						{#if inViewport}
							<PlaylistItem {playlist} visible={inViewport} />
						{/if}
					</div>
				{/each}
			</div>
		{:else if vm.isHorizontal}
			<!-- Horizontal layout -->
			{#each vm.data as albumIndex, index}
				{@const hiddenBySidebar = vm.shouldHideHorizontalItem(index)}
				{@const visibleByFilter = vm.isVisibleByFilter(albumIndex)}
				{@const inViewport = vm.visibleItems.has(index)}
				{@const shouldRender = vm.shouldRenderHorizontalItem(index, albumIndex)}
				<div
					use:vm.observeElement={index}
					class={vm.itemClass(inViewport, hiddenBySidebar, 'flex-shrink-0')}
					style={vm.itemStyle(hiddenBySidebar)}
					style:display={visibleByFilter ? undefined : 'none'}
					onanimationend={() => vm.handleAnimationEnd(index, hiddenBySidebar)}
				>
					{#if shouldRender}
						<AlbumItem {albumIndex} {index} visible={inViewport} isListHovered={isHovered} />
					{/if}
				</div>
			{/each}
		{:else}
			<!-- Grid layout -->
			<div
				class="grid"
				style="grid-template-columns: repeat({vm.state.columnCount}, minmax(0, 1fr));"
			>
				{#each vm.data as albumIndex, index}
					{@const hiddenBySidebar = vm.shouldHideGridItem(index)}
					{@const visibleByFilter = vm.isVisibleByFilter(albumIndex)}
					{@const inViewport = vm.visibleItems.has(index)}
					{@const shouldRender = vm.shouldRenderGridItem(index, albumIndex)}
					<div
						use:vm.observeElement={index}
						class={vm.itemClass(inViewport, hiddenBySidebar)}
						style={vm.itemStyle(hiddenBySidebar)}
						style:display={visibleByFilter ? undefined : 'none'}
						onanimationend={() => vm.handleAnimationEnd(index, hiddenBySidebar)}
					>
						{#if shouldRender}
							<AlbumItem {albumIndex} {index} visible={inViewport} isListHovered={isHovered} />
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
