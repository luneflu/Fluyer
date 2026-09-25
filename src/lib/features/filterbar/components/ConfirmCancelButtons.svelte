<script lang="ts">
	import Icon from '$lib/ui/icon/Icon.svelte';
	import { IconType } from '$lib/ui/icon/types';
	import Button from '$lib/ui/components/Button.svelte';

	interface Props {
		class?: string;
		onconfirm?: () => void;
		oncancel?: () => void;
	}

	const props: Props = $props();
</script>

<div class="grid h-9 w-full grid-cols-[auto_auto] gap-x-2 {props.class}">
	<Button
		class="pointer-events-auto grid h-9 w-full items-center justify-center gap-x-2 rounded p-[3.5px] sm:p-0 sm:px-2"
		onclick={props.onconfirm}
		glassShineColor="rgba(255, 255, 255, 0.3)"
	>
		<div class="w-5">
			<Icon type={IconType.Check} />
		</div>
	</Button>
	<Button
		class="pointer-events-auto grid h-9 w-full items-center justify-center gap-x-2 rounded p-[3.5px] sm:p-0 sm:px-2"
		onclick={props.oncancel}
		glassShineColor="rgba(255, 255, 255, 0.3)"
	>
		<div class="w-5">
			<Icon type={IconType.Cancel} />
		</div>
	</Button>
</div>
