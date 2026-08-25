<script lang="ts">
	import { isMobile, isMacos } from '$lib/platform';
	import Icon from '$lib/ui/icon/Icon.svelte';
	import { IconType } from '$lib/ui/icon/types';
	import View from '$lib/ui/components/View.svelte';
	import ProgressBar from '$lib/ui/components/ProgressBar.svelte';
	import musicStore from '$lib/stores/music.svelte';
	import { MusicConfig } from '$lib/constants/MusicConfig';
	import MusicPlayerService from '$lib/services/MusicPlayerService.svelte';
	import settingStore from '$lib/stores/setting.svelte';
	import { RepeatMode } from '$lib/features/music/types';
	import { usePlayPage } from './viewmodels/usePlayPage.svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';

	const vm = usePlayPage();

	let isFullscreen = $state(false);
	$effect(() => {
		(async () => {
			isFullscreen = isMacos() && (await getCurrentWindow().isFullscreen());
		})();
	});
</script>

<svelte:document
	onkeydown={vm.onKeyDown}
	onmousemove={vm.resetIdleTimer}
	onclick={vm.resetIdleTimer}
/>

{#if settingStore.ui.play.showBackButton}
	<div
		class="absolute left-0 top-0 z-10 hidden ps-3 pt-3 opacity-70 md:block
		{isFullscreen ? '' : 'mt-6'}"
	>
		<button
			id="btn-back"
			class="anim w-7 cursor-pointer {vm.hideBackButton
				? 'hidden'
				: vm.isIdle
					? 'anim-fade-out'
					: 'anim-fade-in'}"
			onclick={vm.handleBackWithDelay}><Icon type={IconType.PlayBack} /></button
		>
	</div>
{/if}
<div
	class="mx-auto grid h-full w-full max-w-[35rem] md:max-w-none md:gap-y-0 md:pt-0
    {vm.lyrics.length > 1
		? 'md:grid-cols-[40%_55%]'
		: 'root-nolyrics justify-center md:grid-cols-[50%]'}
    {isMacos() && 'pt-6'}"
>
	<div
		class="md:col-[1] md:row-[1] {isMobile() ? 'p-5' : 'p-4'} flex items-end md:p-0
        {vm.lyrics.length > 1 ? 'justify-end' : 'justify-center'}"
	>
		<div
			class="w-full sm:w-[90%] md:w-[85%] lg:w-[80%] xl:w-[75%] 2xl:w-[70%] 3xl:w-[65%] {vm.lyrics
				.length > 0 && 'ms-auto'}"
		>
			{#await vm.coverArt}
				<div class="aspect-square w-full"></div>
			{:then image}
				<img
					class="anim anim-fade-in aspect-square w-full rounded-lg object-cover shadow-lg"
					src={image}
					alt="Music Album"
				/>
			{/await}
		</div>
	</div>
	<div
		class="order-last md:order-2 md:col-[1] md:row-[2] {isMobile()
			? 'px-5'
			: 'px-4'} pb-5 pt-2 {isMobile() && 'mb-5'}
        flex md:p-0 md:pb-0 {vm.lyrics.length > 0
			? 'justify-end'
			: 'justify-center'} anim anim-fade-in"
	>
		<View
			class="h-fit w-full rounded-lg
            px-4 py-5 {isMobile() ? '' : 'hover:px-5 hover:py-7'}
            sm:w-[90%] md:mt-4 md:w-[85%] lg:w-[80%] xl:w-[75%] 2xl:w-[70%] 3xl:w-[65%]"
		>
			<div class="grid w-full grid-cols-[auto,1fr,auto]">
				<div class="flex w-12 text-xs xl:text-[13px] 2xl:text-sm">
					<span class="self-end opacity-75">{vm.progressDurationText}</span>
				</div>
				<div
					class="mt-2 overflow-hidden text-center text-sm font-medium opacity-90 sm:text-sm md:text-[15px] xl:text-base"
				>
					<!-- Note: Idk why the title scroll doesn't work without sacrificing first element -->
					<p class="animate-scroll-overflow-text"></p>
					<p class="animate-scroll-overflow-text overflow-x-hidden whitespace-nowrap">
						{vm.music?.albumArtist ?? vm.music?.artist ?? MusicConfig.defaultArtist}
						{MusicConfig.separator}
						{vm.music?.title ?? MusicConfig.defaultTitle}
					</p>
				</div>
				<div class="flex w-12 justify-end text-xs xl:text-[13px] 2xl:text-sm">
					<span class="self-end opacity-75">{vm.progressDurationTotalText}</span>
				</div>
			</div>
			<div class="w-full pb-2 pt-4">
				<ProgressBar
					bind:value={musicStore.progressValue}
					min={MusicConfig.min}
					max={MusicConfig.max}
					step={MusicConfig.step}
					progressPercentage={vm.progressPercentage}
					onProgressClick={vm.handleProgressClick}
					onProgressEnter={vm.handleProgressEnter}
					onProgressMove={vm.handleProgressMove}
					onProgressLeave={vm.handleProgressLeave}
					size="md"
				/>
			</div>
			<div class="mt-4 grid w-full grid-cols-[1fr_auto_auto_auto_1fr] items-center gap-2">
				<div class="flex justify-end">
					{#if settingStore.ui.showRepeatButton}
						<button
							class="mx-2 w-7 md:w-[34px] lg:w-8 {musicStore.repeatMode === RepeatMode.None
								? 'opacity-60'
								: ''}"
							onclick={MusicPlayerService.toggleRepeatMode}
						>
							{#if musicStore.repeatMode === RepeatMode.All}
								<Icon type={IconType.Repeat} />
							{:else if musicStore.repeatMode === RepeatMode.None}
								<Icon type={IconType.RepeatPlayNone} />
							{:else if musicStore.repeatMode === RepeatMode.One}
								<Icon type={IconType.RepeatOne} />
							{/if}
						</button>
					{/if}
				</div>
				<div class="flex justify-end">
					<button
						class="w-12 sm:w-10 md:w-12 lg:w-[3.25rem] xl:w-14"
						onclick={vm.handleButtonPrevious}><Icon type={IconType.Previous} /></button
					>
				</div>
				<div class="flex justify-center">
					<button
						class="w-12 sm:w-10 md:w-12 lg:w-[3.25rem] xl:w-14"
						onclick={vm.handleButtonPlayPause}
					>
						{#if musicStore.isPlaying}
							<Icon type={IconType.Pause} />
						{:else}
							<Icon type={IconType.Play} />
						{/if}
					</button>
				</div>
				<div class="flex justify-start">
					<button class="w-12 sm:w-10 md:w-12 lg:w-[3.25rem] xl:w-14" onclick={vm.handleButtonNext}
						><Icon type={IconType.Next} /></button
					>
				</div>
				<div class="flex justify-start">
					{#if settingStore.ui.showShuffleButton}
						<button
							class="mx-2 w-7 md:w-[34px] lg:w-8 {musicStore.isShuffled
								? 'text-primary'
								: 'opacity-60'}"
							onclick={vm.handleButtonShuffle}
						>
							<Icon type={IconType.Shuffle} />
						</button>
					{/if}
				</div>
			</div>
			{#if settingStore.ui.play.showVolume && !settingStore.bitPerfectMode}
				<div id="volume-bar" class="mt-5">
					<div class="grid grid-cols-[auto_1fr_auto] items-center gap-3">
						<button class="w-5" onclick={() => (musicStore.volume = 0)}>
							<Icon type={IconType.Mute} />
						</button>
						<div class="relative">
							<ProgressBar
								bind:value={musicStore.volume}
								progressPercentage={vm.volumePercentage}
								onProgressClick={vm.handleVolumeProgressClick}
								min={MusicConfig.vmin}
								max={MusicConfig.vmax}
								step={MusicConfig.vstep}
								showTooltip={false}
								size="sm"
							/>
						</div>
						<button class="w-5" onclick={() => (musicStore.volume = 1)}>
							<Icon type={IconType.Speaker} />
						</button>
					</div>
				</div>
			{/if}
		</View>
	</div>
	{#if vm.lyrics.length > 0}
		<div
			class="scrollbar-hidden anim anim-fast anim-fade-in-up w-full overflow-y-auto overflow-x-hidden
            [mask-image:linear-gradient(to_bottom,rgba(0,0,0,1)_60%,rgba(0,0,0,0))]
            md:col-[2] md:row-[1/span_2]
            md:h-screen md:px-20 md:[mask-image:linear-gradient(to_bottom,rgba(0,0,0,0),rgba(0,0,0,1),rgba(0,0,0,0))] {isMobile()
				? 'px-5'
				: 'px-4'}"
			bind:this={vm.lyricContainerElement}
		>
			<div class="flex">
				<div
					id="lyrics"
					class="h-full w-full text-[1.15rem] font-bold sm:text-[1.25rem]
                    md:my-[40vh]
					md:w-[55vw] md:text-[1.4rem] lg:text-[1.5rem] xl:text-[1.7rem]"
					style="padding-bottom: {window.innerWidth < 768
						? vm.lyricContainerElement?.clientHeight - 60
						: 0}px"
				>
					{#each vm.lyrics as lyric, i}
						<div
							id={vm.selectedLyricIndex === i ? 'selected-lyric' : ''}
							class={vm.selectedLyricIndex === i
								? 'py-5 text-[1.30rem] sm:text-[1.40rem] md:py-7 md:text-[1.55rem] lg:text-[1.65rem] xl:text-[1.85rem]'
								: 'py-5 opacity-50 md:py-7 lg:py-10'}
						>
							{#if lyric.value.length > 0}
								{lyric.value}
							{:else}
								<div
									class={vm.selectedLyricIndex === i
										? 'w-[1.4rem] md:w-[1.9rem] lg:w-[2.25rem]'
										: 'w-[1.25rem] md:w-[1.75rem] lg:w-[2.15rem]'}
								>
									<Icon type={IconType.Note} />
								</div>
							{/if}
						</div>
					{/each}
				</div>
			</div>
		</div>
	{/if}
</div>
