<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';
	import { IconType } from '$lib/ui/icon/types';
	import Icon from '$lib/ui/icon/Icon.svelte';
	import { onMount } from 'svelte';
	import SettingLabel from '$lib/features/settings/SettingLabel.svelte';
	import SettingInput from '$lib/features/settings/SettingInput.svelte';
	import { isAndroid, isDesktop } from '$lib/platform';
	import TauriIntroAPI from '$lib/tauri/TauriIntroAPI';
	import PersistentStoreService from '$lib/services/PersistentStoreService.svelte.js';
	import LibraryService from '$lib/services/LibraryService.svelte.js';
	import ToastService from '$lib/services/ToastService.svelte.js';

	let musicPath = $state<string[]>([]);
	let isLoading = $state(false);

	async function refreshPath() {
		musicPath = await PersistentStoreService.musicPath.get();
	}

	async function addPath() {
		let newPath: string | null = null;
		if (isDesktop()) {
			newPath = (await open({
				directory: true,
				multiple: false
			})) as string | null;
		}
		if (isAndroid()) {
			newPath = await TauriIntroAPI.requestDirectoryPath();
		}
		if (!newPath || musicPath.includes(newPath)) return;

		isLoading = true;
		await PersistentStoreService.musicPath.add(newPath);
		await refreshPath();
		await LibraryService.loadMusicList();
		isLoading = false;

		ToastService.info('Music path added');
	}

	async function removePath(index: number) {
		isLoading = true;
		await PersistentStoreService.musicPath.remove(index);
		await refreshPath();
		await LibraryService.loadMusicList();
		isLoading = false;

		ToastService.info('Music path removed');
	}

	onMount(refreshPath);
</script>

<SettingLabel
	title="Music Library Paths"
	description="Directories where your music files are stored."
/>

{#each musicPath as path, index}
	<SettingInput>
		<div class="grid w-full grid-cols-[auto_min-content] px-3 py-2">
			<input class="w-full bg-transparent" value={path} readonly />
			{#if musicPath.length > 1}
				<button
					class="flex h-6 w-6 cursor-pointer items-center justify-center rounded"
					onclick={() => removePath(index)}
				>
					<Icon type={IconType.TrashRed} />
				</button>
			{/if}
		</div>
	</SettingInput>
{/each}
<SettingInput>
	<div class="grid w-full cursor-pointer grid-cols-[auto_min-content] px-3 py-2">
		<input
			class="text-opacity-background-70 w-full cursor-pointer bg-transparent"
			value="Add new music path..."
			readonly
			disabled={isLoading}
			onclick={addPath}
		/>
	</div>
</SettingInput>
