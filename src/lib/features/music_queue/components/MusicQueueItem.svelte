<script lang="ts">
	interface Props {
		index: number;
		visible?: boolean;
	}

	import Icon from '$lib/ui/icon/Icon.svelte';
	import { IconType } from '$lib/ui/icon/types';
	import { useMusicQueueItem } from '$lib/features/music_queue/viewmodels/useMusicQueueItem.svelte';

	let { index, visible = true }: Props = $props();

	// Initialize viewmodel
	const vm = useMusicQueueItem(
		() => index,
		() => visible
	);
</script>

<div class="relative">
	<div class="relative grid grid-cols-[max-content_auto_max-content] px-3 py-2">
		<div class="h-12 w-12 md:h-14 md:w-14">
			{#await vm.coverArt}
				<div class="aspect-square w-full"></div>
			{:then image}
				{#if image}
					<img
						class="animate__animated animate__fadeIn aspect-square w-full rounded"
						src={image}
						alt="Album"
					/>
				{/if}
			{/await}
		</div>
		<div class="ms-3 text-sm md:text-base">
			<p class="font-medium">{vm.music?.title ?? '...'}</p>
			<p class="text-opacity-background-80">{vm.music?.artist ?? '...'}</p>
		</div>
		<div class="h-12 w-12 md:h-14 md:w-14"></div>
	</div>
	{#if vm.isPlaying}
		<div
			class="absolute left-0 top-0 z-10 grid w-full grid-cols-[max-content_auto_max-content] px-3 py-2"
		>
			<div class="aspect-square h-12 w-12 md:h-14 md:w-14"></div>
			<div></div>
			<div class="aspect-square h-11 w-11 p-1 md:h-12 md:w-12 lg:h-14 lg:w-14 lg:p-3">
				<div class="animate__animated animate__infinite animate__pulse w-3/4 md:w-full">
					<Icon type={IconType.Playing} />
				</div>
			</div>
		</div>
	{:else}
		<div
			class="playlist-item-controls absolute left-0 top-0 z-10 grid w-full grid-cols-[max-content_auto_max-content] px-3 py-2"
		>
			<button
				class="aspect-square h-12 w-12 md:h-14 md:w-14"
				onclick={vm.goToPlaylist}
				onpointerdown={(e) => e.stopPropagation()}
			>
				{#if !vm.isPlaying}
					<div class="h-full w-full rounded bg-black bg-opacity-40 lg:p-1">
						{#if vm.isPrevious}
							<Icon type={IconType.Previous} />
						{:else}
							<Icon type={IconType.Next} />
						{/if}
					</div>
				{/if}
			</button>
			<div class="cursor-grab"></div>
			<button
				class="aspect-square h-12 w-12 md:h-14 md:w-14 lg:p-1"
				onclick={vm.removePlaylist}
				onpointerdown={(e) => e.stopPropagation()}
			>
				<Icon type={IconType.Remove} />
			</button>
		</div>
	{/if}
</div>

<style lang="scss">
	.playlist-item-controls {
		opacity: 0;

		&:hover {
			animation-name: fadeIn;
			animation-duration: 0.5s;
			animation-fill-mode: forwards;
		}
	}
</style>
