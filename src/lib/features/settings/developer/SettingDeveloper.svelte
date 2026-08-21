<script lang="ts">
	import SettingLabel from '$lib/features/settings/SettingLabel.svelte';
	import SettingInput from '$lib/features/settings/SettingInput.svelte';
	import SettingButton from '$lib/features/settings/SettingButton.svelte';
	import { IconType } from '$lib/ui/icon/types';
	import { isDesktop } from '$lib/platform';
	import settingStore from '$lib/stores/setting.svelte.js';
	import PersistentStoreService from '$lib/services/PersistentStoreService.svelte.js';
	import ToastService from '$lib/services/ToastService.svelte.js';
	import TauriDeveloperAPI from '$lib/tauri/TauriDeveloperAPI';
	import { relaunch } from '@tauri-apps/plugin-process';

	function onDeveloperModeChange(
		e: Event & {
			currentTarget: EventTarget & HTMLInputElement;
		}
	) {
		settingStore.developerMode = e.currentTarget.checked;
		PersistentStoreService.developerMode.set(e.currentTarget.checked);
		ToastService.info(`Developer mode is ${e.currentTarget.checked ? 'enabled' : 'disabled'}`);
	}

	async function clearData() {
		await TauriDeveloperAPI.clearDeveloperData();
	}

	async function clearCache() {
		await TauriDeveloperAPI.clearDeveloperCache();
	}

	async function saveLog() {
		await TauriDeveloperAPI.saveDeveloperLog();
	}
</script>

<SettingLabel title="Developer" description="Logging and debugging purposes." />

<SettingInput>
	<label class="grid cursor-pointer grid-cols-[min-content_auto] items-center gap-3 px-3 py-2">
		<input
			type="checkbox"
			class="h-4 w-4"
			checked={settingStore.developerMode}
			onchange={onDeveloperModeChange}
		/>
		<div>Developer Mode</div>
	</label>
</SettingInput>
	<SettingButton label="Clear Data & Cache" icon={IconType.Trash} onclick={clearData} />
	<SettingButton label="Clear Cache" icon={IconType.Trash} onclick={clearCache} />
{#if settingStore.developerMode && isDesktop()}
	<SettingButton label="Save Log" icon={IconType.SaveLog} onclick={saveLog} />
{/if}
