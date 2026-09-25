<script lang="ts">
	import { IconType } from '$lib/ui/icon/types';
	import { MusicConfig } from '$lib/constants/MusicConfig';
	import musicStore from '$lib/stores/music.svelte';
	import settingStore from '$lib/stores/setting.svelte';
	import ProgressBar from '$lib/ui/components/ProgressBar.svelte';
	import Icon from '$lib/ui/icon/Icon.svelte';
	import View from '$lib/ui/components/View.svelte';

	let volumePercentage = $derived(musicStore.volume * 100);

	function handleVolumeProgressClick(percentage: number) {
		musicStore.volume = percentage / 100;
	}

	function handleMuteButton() {
		musicStore.volume = 0;
	}

	function handleSpeakerButton() {
		musicStore.volume = 1;
	}
</script>

<View
	class="mx-2 mt-4 grid cursor-default
    grid-cols-[min-content_auto_min-content] items-center gap-3 rounded px-3 py-3 text-base font-medium tracking-wide
    md:hidden md:text-lg {settingStore.bitPerfectMode ? 'pointer-events-none opacity-50' : ''}"
>
	<button class="w-5" onclick={handleMuteButton}>
		<Icon type={IconType.Mute} />
	</button>
	<div class="relative w-full">
		<ProgressBar
			bind:value={musicStore.volume}
			progressPercentage={volumePercentage}
			onProgressClick={handleVolumeProgressClick}
			min={MusicConfig.vmin}
			max={MusicConfig.vmax}
			step={MusicConfig.vstep}
			showTooltip={false}
			class="w-full"
			size="sm"
		/>
	</div>
	<button class="w-5" onclick={handleSpeakerButton}>
		<Icon type={IconType.Speaker} />
	</button>
</View>
