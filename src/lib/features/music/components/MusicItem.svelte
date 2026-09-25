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

<div class="group w-full text-sm md:text-base">
	<div class="grid grid-cols-[max-content_auto_max-content] py-2">
		<!-- Image / folder icon cell -->
		{#await vm.coverArt}
			<div class="aspect-square h-12 w-12 md:h-14 md:w-14"></div>
		{:then image}
			{#if image && !folder}
				<div class="relative h-12 w-12 md:h-14 md:w-14">
					<img
						class={vm.isImageAnimating
							? 'anim anim-fade-in absolute inset-0 h-full w-full rounded object-cover'
							: 'absolute inset-0 h-full w-full rounded object-cover'}
						src={image}
						alt="Album"
						onanimationend={() => (vm.isImageAnimating = false)}
					/>
					{#if !playlistStore.isCreating}
						<button
							class="absolute inset-0 grid items-center justify-items-center rounded bg-black bg-opacity-40 opacity-0 transition-opacity duration-500 group-hover:opacity-100 md:p-1"
							onclick={vm.addMusicAndPlay}
						>
							<Icon type={IconType.Play} />
						</button>
					{/if}
				</div>
			{:else if image && folder}
				<!-- Folder with album art -->
				<div
					class={vm.isImageAnimating
						? 'anim anim-fade-in relative aspect-square h-12 w-12 transition-transform duration-300 group-hover:scale-110 md:h-14 md:w-14'
						: 'relative aspect-square h-12 w-12 transition-transform duration-300 group-hover:scale-110 md:h-14 md:w-14'}
					onanimationend={() => (vm.isImageAnimating = false)}
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
				<div class="aspect-square h-12 w-12 md:h-14 md:w-14"></div>
			{/if}
		{/await}

		<!-- Text cell — click target for play/folder -->
		<div
			class="ms-3 cursor-pointer overflow-hidden"
			onclick={folder ? vm.selectFolder : vm.addMusicAndPlay}
		>
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

		<!-- Action cell (third column) -->
		<div class="h-12 w-12 ps-2 md:h-14 md:w-14">
			{#if playlistStore.isCreating && vm.resolvedMusic}
				<label
					class="flex aspect-square h-full w-full cursor-pointer items-center justify-center ps-2"
				>
					<input
						type="checkbox"
						checked={vm.isSelectedForPlaylist}
						onchange={vm.togglePlaylistSelection}
						class="h-5 w-5 accent-white"
					/>
				</label>
			{:else}
				<button
					class="aspect-square h-full w-full opacity-0 transition-opacity duration-700 group-hover:opacity-100"
					onclick={vm.addMusic}
				>
					<Icon type={IconType.QueueMusic} />
				</button>
			{/if}
		</div>
	</div>
</div>
