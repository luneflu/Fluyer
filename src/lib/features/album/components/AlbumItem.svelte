<script lang="ts">
	import { useAlbumItem } from '$lib/features/album/viewmodels/useAlbumItem.svelte';

	interface Props {
		albumIndex: number;
		index: number;
		visible?: boolean;
	}

	let { albumIndex, index, visible = false }: Props = $props();

	const vm = useAlbumItem(
		() => albumIndex,
		() => index,
		() => visible
	);
</script>

<div class="col-auto row-[1] h-fit px-3 pb-3">
	<div class="relative w-full">
		<div
			class="absolute left-0 top-0 w-full h-full cursor-pointer rounded-lg border-2 border-white transition-all
            {vm.isValidFilterAlbum ? 'z-10' : 'album-item-actions z-20 bg-white/20'}"
			onclick={!vm.isValidFilterAlbum ? vm.setFilterAlbum : undefined}
			ondblclick={vm.playAlbum}
		></div>
		{#await vm.coverArt}
			<div class="aspect-square w-full"></div>
		{:then image}
			{#if image}
				<img
					class="animate__animated animate__fadeIn aspect-square w-full rounded-lg object-cover"
					src={image}
					alt="Album"
				/>
			{:else}
				<div class="aspect-square w-full rounded-lg"></div>
			{/if}
		{/await}
	</div>
	<p
		class="animate-scroll-overflow-text mt-2 overflow-hidden whitespace-nowrap font-medium md:text-lg"
	>
		{vm.music?.album}
	</p>
	<p
		class="text-opacity-background-80 animate-scroll-overflow-text overflow-hidden whitespace-nowrap text-[15px] md:text-base"
	>
		{vm.music?.albumArtist ?? vm.music?.artist}
	</p>
</div>

<style lang="scss">
	.album-item-actions {
		opacity: 0;
		transition: opacity 0.75s;

		&:hover {
			opacity: 1;
			transition: opacity 0.5s;
		}
	}
</style>
