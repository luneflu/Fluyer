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

	let imageAnimating = $state(false);
</script>

<div class="col-auto row-[1] h-fit px-3 pb-3">
	<div class="group relative w-full">
		{#await vm.coverArt}
			<div class="aspect-square w-full"></div>
		{:then image}
			{#if image}
				<img
					class={imageAnimating
						? 'anim anim-fade-in aspect-square w-full rounded object-cover'
						: 'aspect-square w-full rounded object-cover'}
					src={image}
					alt="Album"
					onload={() => (imageAnimating = true)}
					onanimationend={() => (imageAnimating = false)}
				/>
			{:else}
				<div class="aspect-square w-full rounded"></div>
			{/if}
		{/await}
		<div
			class="absolute left-0 top-0 h-full w-full cursor-pointer rounded border-2 border-white transition-all
            {vm.isValidFilterAlbum ? 'z-10' : 'z-20 bg-white/20 opacity-0 transition-opacity duration-700 group-hover:opacity-100'}"
			onclick={!vm.isValidFilterAlbum ? vm.setFilterAlbum : undefined}
			ondblclick={vm.playAlbum}
		></div>
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
