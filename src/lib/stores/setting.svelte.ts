import { SettingAnimatedBackgroundType } from '$lib/features/settings/animated_background/types';

const settingStore = $state({
	animatedBackground: {
		trigger: '',
		type: SettingAnimatedBackgroundType.Pallete
	},

	ui: {
		showRepeatButton: true,
		showShuffleButton: true,
		play: {
			showBackButton: true,
			showVolume: false
		}
	},

	developerMode: false,
	bitPerfectMode: false
});

export default settingStore;
