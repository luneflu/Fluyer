import { Store } from '@tauri-apps/plugin-store';
import { SettingAnimatedBackgroundType } from '$lib/features/settings/animated_background/types';
import { IconThemeType } from '$lib/ui/icon/types';
import musicStore from '$lib/stores/music.svelte';
import settingStore from '$lib/stores/setting.svelte';
import mobileStore from '$lib/stores/mobile.svelte';
import iconStore from '$lib/stores/icon.svelte';
import equalizerStore from '$lib/stores/equalizer.svelte';

const STORE_PATH = 'store.json';

const getStore = async () => Store.load(STORE_PATH, { autoSave: true });
const makeGetter =
	<T>(key: string, fallback?: T) =>
	async () =>
		(await (await getStore()).get<T>(key)) ?? fallback;
const makeSetter =
	<T>(key: string) =>
	async (value: T) =>
		(await getStore()).set(key, value);
const makeBinding = <T>(key: string, fallback: T, setStoreFn: (v: T) => void) => ({
	// @ts-ignore
	initialize: async () => setStoreFn(await makeGetter<T>(key, fallback)()),
	get: makeGetter<T>(key, fallback),
	set: makeSetter<T>(key)
});

const PersistentStoreService = {
	get: getStore,

	initialize: async () => {
		await Promise.all([
			PersistentStoreService.animatedBackgroundType.initialize(),
			PersistentStoreService.iconTheme.initialize(),
			PersistentStoreService.developerMode.initialize(),
			PersistentStoreService.userInterface.play.showBackButton.initialize(),
			PersistentStoreService.userInterface.play.showVolume.initialize(),
			PersistentStoreService.userInterface.showRepeatButton.initialize(),
			PersistentStoreService.userInterface.showShuffleButton.initialize(),
			PersistentStoreService.swipeGuide.initialize(),
			PersistentStoreService.equalizer.initialize(),
			PersistentStoreService.bitPerfectMode.initialize(),
			PersistentStoreService.discordRpcEnabled.initialize(),
			PersistentStoreService.volume.initialize()
		]);
	},

	musicPath: {
		key: 'music-path',
		separator: '||',
		get: async () => (await makeGetter<string>('music-path')())?.split('||') ?? [],
		set: async (value: string[]) => (await getStore()).set('music-path', value.join('||')),
		add: async (value: string) => {
			const paths = await PersistentStoreService.musicPath.get();
			paths.push(value);
			await PersistentStoreService.musicPath.set(paths);
		},
		remove: async (index: number) => {
			const paths = await PersistentStoreService.musicPath.get();
			paths.splice(index, 1);
			await PersistentStoreService.musicPath.set(paths);
		}
	},

	volume: makeBinding<number>('volume', 1, (value) => (musicStore.volume = value)),

	animatedBackgroundType: makeBinding<SettingAnimatedBackgroundType>(
		'animated-background-type',
		SettingAnimatedBackgroundType.Prominent,
		(value) => (settingStore.animatedBackground.type = value)
	),

	developerMode: makeBinding(
		'developer-mode',
		false,
		(value) => (settingStore.developerMode = value)
	),

	iconTheme: makeBinding(
		'icon-theme',
		IconThemeType.Phosphor,
		(value) => (iconStore.theme = value)
	),

	bitPerfectMode: makeBinding(
		'bit-perfect-mode',
		false,
		(value) => (settingStore.bitPerfectMode = value)
	),

	discordRpcEnabled: makeBinding(
		'discord-rpc-enabled',
		true,
		(value) => (settingStore.discordRpcEnabled = value)
	),

	swipeGuide: makeBinding('swipe-guide', true, (value) => (mobileStore.showSwipeGuide = value)),

	userInterface: {
		showRepeatButton: makeBinding(
			'ui-show-repeat-button',
			true,
			(value) => (settingStore.ui.showRepeatButton = value)
		),
		showShuffleButton: makeBinding(
			'ui-show-shuffle-button',
			true,
			(value) => (settingStore.ui.showShuffleButton = value)
		),
		play: {
			showBackButton: makeBinding(
				'ui-play-show-back-button',
				true,
				(value) => (settingStore.ui.play.showBackButton = value)
			),
			showVolume: makeBinding(
				'ui-play-show-volume',
				false,
				(value) => (settingStore.ui.play.showVolume = value)
			)
		}
	},

	equalizer: makeBinding<number[]>(
		'equalizer',
		Array(18).fill(0),
		(value) => (equalizerStore.values = value)
	),

	skippedUpdateVersion: makeBinding<string>('skipped-update-version', '', (value) => {})
};

export default PersistentStoreService;
