<script lang="ts">
	import Toggle from '$lib/ui/components/Toggle.svelte';
	import { IconType } from '$lib/ui/icon/types';

	interface ToggleOption {
		value: string | number;
		icon: IconType;
		label?: string;
	}

	interface Props {
		class?: string;
		iconStyle?: string;
		options: ToggleOption[];
		selected: string | number;
		onchange?: (value: string | number) => void;
	}

	const props: Props = $props();
</script>

<Toggle
	class="pointer-events-auto h-9 w-full {props.class}"
	iconStyle={props.iconStyle}
	options={props.options}
	selected={props.selected}
	onchange={props.onchange}
	glassShineColor="rgba(255, 255, 255, 0.3)"
/>
