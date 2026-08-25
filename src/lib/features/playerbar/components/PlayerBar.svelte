<script lang="ts">
	import { IconType } from '$lib/ui/icon/types';
	import { MusicConfig } from '$lib/constants/MusicConfig';
	import { RepeatMode } from '$lib/features/music/types';
	import musicStore from '$lib/stores/music.svelte';
	import settingStore from '$lib/stores/setting.svelte';
	import MusicPlayerService from '$lib/services/MusicPlayerService.svelte';
	import ProgressService from '$lib/services/ProgressService.svelte';
	import mobileStore from '$lib/stores/mobile.svelte';
	import ProgressBar from '$lib/ui/components/ProgressBar.svelte';
	import View from '$lib/ui/components/View.svelte';
	import Icon from '$lib/ui/icon/Icon.svelte';
	import { usePlayerBar } from '../viewmodels/usePlayerBar.svelte';
	import { isMobile } from '$lib/platform';

	interface Props {
		tooltipVisible?: boolean;
	}

	let { tooltipVisible = $bindable(false) }: Props = $props();

	const vm = usePlayerBar();
</script>

<svelte:window onresize={vm.updatePlayerBarHeight} />

<div
	class="anim anim-slide-in-up w-full px-3 pt-3"
	style="padding-bottom: {mobileStore.navigationBarHeight > 12
		? mobileStore.navigationBarHeight
		: 12}px;"
	bind:this={vm.element}
>
	<ProgressBar
		bind:value={musicStore.progressValue}
		progressPercentage={vm.progressPercentage}
		onProgressClick={vm.handleProgressClick}
		min={MusicConfig.min}
		max={MusicConfig.max}
		step={MusicConfig.step}
		showTooltip={true}
		bind:tooltipVisible
		tooltipFormatter={(percentage) =>
			ProgressService.formatDuration((musicStore.currentMusic?.duration ?? 0) * (percentage / 100))}
		class="mb-3"
		size="lg"
	/>

	<View class="rounded-full px-3 py-2 {isMobile() ? '' : 'hover:px-4 hover:py-3'}"
		glassShineSize="xs">
		<div class="grid w-full grid-cols-[auto_min-content] py-1 md:grid-cols-3">
			<div class="flex items-center ps-1 sm:gap-x-1">
				<button class="hidden w-10 sm:block md:w-12 lg:w-12" onclick={vm.handleButtonPrevious}
					><Icon type={IconType.Previous} /></button
				>
				<button class="w-10 md:w-12 lg:w-12" onclick={vm.handleButtonPlayPause}>
					{#if vm.isPlaying}
						<Icon type={IconType.Pause} />
					{:else}
						<Icon type={IconType.Play} />
					{/if}
				</button>
				<button class="hidden w-10 sm:block md:w-12 lg:w-12" onclick={vm.handleButtonNext}
					><Icon type={IconType.Next} /></button
				>
			</div>
			<div
				class="order-first ms-2 items-center justify-center text-sm
                md:order-none md:ms-0
                md:flex md:text-[15px]"
			>
				<div class="grid grid-cols-[2.5rem_auto] md:grid-cols-[3rem_auto]">
					<button onclick={vm.redirectToPlay}>
						{#await vm.coverArt}
							<div class="aspect-square w-full"></div>
						{:then image}
							<img
								class="anim anim-fade-in aspect-square w-full rounded object-cover"
								src={image}
								alt="Album"
							/>
						{/await}
					</button>
					<div class="ms-3 grid grid-rows-[auto_1fr_1fr] overflow-hidden">
						<!-- Note: Idk why the title scroll doesn't work without sacrificing first element -->
						<p class="animate-scroll-overflow-text"></p>
						<p class="animate-scroll-overflow-text overflow-x-hidden whitespace-nowrap font-medium">
							{musicStore.currentMusic?.title ?? MusicConfig.defaultTitle}
						</p>
						<p
							class="animate-scroll-overflow-text overflow-x-hidden whitespace-nowrap text-opacity-80"
						>
							{musicStore.currentMusic?.artist ?? MusicConfig.defaultArtist}
						</p>
					</div>
				</div>
			</div>
			<div class="hidden justify-end md:grid">
				<div class="grid items-center gap-3 {vm.gridRight}">
					{#if settingStore.ui.showRepeatButton}
						<button
							class="w-6 {musicStore.repeatMode === RepeatMode.None ? 'opacity-60' : ''}"
							onclick={MusicPlayerService.toggleRepeatMode}
						>
							{#if musicStore.repeatMode === RepeatMode.All}
								<Icon type={IconType.Repeat} />
							{:else if musicStore.repeatMode === RepeatMode.None}
								<Icon type={IconType.RepeatNone} />
							{:else if musicStore.repeatMode === RepeatMode.One}
								<Icon type={IconType.RepeatOne} />
							{/if}
						</button>
					{/if}
					{#if settingStore.ui.showShuffleButton}
						<button
							class="w-6 {musicStore.isShuffled ? 'text-primary' : 'opacity-60'}"
							onclick={vm.handleButtonShuffle}
						>
							<Icon type={IconType.Shuffle} />
						</button>
					{/if}
					<button
						class="w-6 {settingStore.bitPerfectMode ? 'pointer-events-none' : ''}"
						onclick={vm.handleVolumeButton}
					>
						{#if vm.volumePercentage > 0}
							<Icon type={IconType.Speaker} />
						{:else}
							<Icon type={IconType.Mute} />
						{/if}
					</button>
					<div class="relative w-24 {settingStore.bitPerfectMode ? 'pointer-events-none' : ''}">
						<ProgressBar
							bind:value={musicStore.volume}
							progressPercentage={vm.volumePercentage}
							onProgressClick={vm.handleVolumeProgressClick}
							min={MusicConfig.vmin}
							max={MusicConfig.vmax}
							step={MusicConfig.vstep}
							showTooltip={false}
							class="w-24"
							size="sm"
						/>
					</div>
				</div>
			</div>
		</div>
	</View>
</div>
