<script lang="ts">
	import { type MusicData, type FolderData } from './types';
	import { useMusicItem } from '../viewmodels/useMusicItem.svelte';
	import Icon from '$lib/ui/icon/Icon.svelte';
	import { IconType } from '$lib/ui/icon/types';
	import playlistStore from '$lib/stores/playlist.svelte';

	interface Props {
		musicIndex?: number;
		music?: MusicData;
		folder?: FolderData;
		visible?: boolean;
	}

	let { musicIndex, music: musicProp, folder, visible = true }: Props = $props();

	const vm = useMusicItem(
		() => musicIndex,
		() => musicProp,
		() => folder,
		() => visible
	);
</script>

<div class="group relative w-full text-sm md:text-base">
	<div class="grid grid-cols-[max-content_auto_max-content] py-2">
		{#await vm.coverArt}
			<div class="relative aspect-square h-12 w-12 md:h-14 md:w-14"></div>
		{:then image}
			{#if image && !folder}
				<img
					class="anim anim-fade-in relative h-12 w-12 rounded object-cover md:h-14 md:w-14"
					src={image}
					alt="Album"
				/>
			{:else if image && folder}
				<!-- Folder with album art -->
				<div
					class="anim anim-fade-in relative aspect-square h-12 w-12 transition-transform duration-300 group-hover:scale-110 md:h-14 md:w-14"
				>
					<div class="absolute inset-0 opacity-75">
						<Icon type={IconType.Folder} />
					</div>
					<div class="absolute inset-0 flex items-center justify-center">
						<img
							class="mt-2 h-4 w-4 rounded-sm object-cover shadow-md md:h-5 md:w-5"
							src={image}
							alt="Album"
						/>
					</div>
				</div>
			{:else}
				<div class="relative aspect-square h-12 w-12 md:h-14 md:w-14"></div>
			{/if}
		{/await}

		<div class="ms-3 overflow-hidden">
			<p
				class="animate-scroll-overflow-text overflow-hidden whitespace-nowrap text-sm/[14px] font-medium md:text-sm"
			>
				{vm.titleLabel}
			</p>
			<p
				class="text-opacity-background-90 animate-scroll-overflow-text overflow-hidden whitespace-nowrap pt-[4px] text-xs/[14px] md:pt-0 md:text-xs"
			>
				{vm.mediumLabel}
			</p>
			<p class="text-opacity-background-90 mt-[2px] text-xs/[14px] md:text-xs">
				{vm.smallLabel}
			</p>
		</div>

		<div class="h-12 w-12 ps-2 md:h-14 md:w-14"></div>
	</div>

	<div class="absolute left-0 top-0 w-full py-2">
		{#if playlistStore.isCreating && vm.resolvedMusic}
			<div class="grid w-full grid-cols-[max-content_auto_max-content]">
				<div class="h-12 w-12 md:h-14 md:w-14"></div>

				<div class="cursor-pointer" onclick={vm.togglePlaylistSelection}></div>

				<div class="h-12 w-12 ps-4 md:h-14 md:w-14">
					<label
						class="flex aspect-square h-full w-full cursor-pointer items-center justify-center"
					>
						<input
							type="checkbox"
							checked={vm.isSelectedForPlaylist}
							onchange={vm.togglePlaylistSelection}
							class="h-5 w-5 accent-white"
						/>
					</label>
				</div>
			</div>
		{:else}
			<div class="music-item-play grid w-full grid-cols-[max-content_auto_max-content]">
				<button
					class="h-12 w-12 md:h-14 md:w-14"
					onclick={folder ? vm.selectFolder : vm.addMusicAndPlay}
				>
					{#if !folder}
						<div
							class="box-border grid items-center justify-items-center rounded bg-black bg-opacity-40 md:p-1"
						>
							<Icon type={IconType.Play} />
						</div>
					{/if}
				</button>

				<div class="cursor-pointer" onclick={folder ? vm.selectFolder : vm.addMusicAndPlay}></div>

				<div class="h-12 w-12 ps-4 md:h-14 md:w-14">
					<button class="aspect-square h-full w-full" onclick={vm.addMusic}>
						<Icon type={IconType.QueueMusic} />
					</button>
				</div>
			</div>
		{/if}
	</div>
</div>

<style lang="scss">
	.music-item-play {
		opacity: 0;
		transition: opacity 0.75s;

		&:hover {
			opacity: 1;
			transition: opacity 0.5s;
		}
	}
</style>
