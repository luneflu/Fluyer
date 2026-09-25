<script lang="ts">
	import Icon from '$lib/ui/icon/Icon.svelte';
	import { IconType } from '$lib/ui/icon/types';
	import Button from '$lib/ui/components/Button.svelte';

	interface Props {
		onclick?: () => void;
	}

	const props: Props = $props();
</script>

<Button
	class="pointer-events-auto grid aspect-square h-9 shrink-0 items-center justify-center rounded"
	onclick={props.onclick}
	glassShineColor="rgba(255, 255, 255, 0.3)"
>
	<div class="w-5">
		<Icon type={IconType.PlaylistAdd} />
	</div>
</Button>
