<script lang="ts">
	interface Props {
		value: number;
		min?: number;
		max?: number;
		step?: number;
		progressPercentage: number;
		showTooltip?: boolean;
		tooltipFormatter?: (percentage: number) => string;
		class?: string;
		onValueChange?: (value: number) => void;
		onProgressClick?: (percentage: number) => void;
		onProgressEnter?: (percentage: number) => void;
		onProgressMove?: (percentage: number) => void;
		onProgressLeave?: (percentage: number) => void;
		size?: 'sm' | 'md' | 'lg';
		tooltipVisible?: boolean;
	}

	let {
		value = $bindable(),
		min = 0,
		max = 100,
		step = 1,
		progressPercentage,
		showTooltip = false,
		tooltipFormatter = (percentage: number) => `${percentage.toFixed(0)}%`,
		class: className = '',
		onValueChange,
		onProgressClick,
		onProgressEnter,
		onProgressMove,
		onProgressLeave,
		size = 'md',
		tooltipVisible = $bindable(false)
	}: Props = $props();

	const tooltipMargin = 0;

	let tooltip: HTMLDivElement;
	let tooltipPosition = $state(0);
	let tooltipText = $state('0:00');
	let touchLastX = $state(0);
	let containerWidth = $state(0);
	let container: HTMLDivElement;

	function setTooltipVisible(visible: boolean, percentage?: number) {
		if (!showTooltip) return;
		if (visible && percentage !== undefined) {
			tooltipText = tooltipFormatter(percentage);
		}
		tooltipVisible = visible;
	}

	function getPointerPercentage(x: number) {
		if (tooltip) {
			tooltipPosition = x - tooltip.offsetWidth / 2;

			if (tooltipPosition < tooltipMargin) tooltipPosition = tooltipMargin;
			else if (tooltipPosition + tooltip.offsetWidth > containerWidth - tooltipMargin)
				tooltipPosition = containerWidth - tooltip.offsetWidth - tooltipMargin;
		}

		return (x / containerWidth) * 100;
	}

	function getEventPercentage(e: MouseEvent | TouchEvent) {
		updateContainerWidth();

		const rect = container.getBoundingClientRect();
		const x = 'touches' in e ? e.touches[0].clientX - rect.left : e.clientX - rect.left;

		if ('touches' in e) touchLastX = x;

		return getPointerPercentage(x);
	}

	// Shared event handlers
	function handleEnter(e: MouseEvent | TouchEvent) {
		const percentage = getEventPercentage(e);
		onProgressEnter?.(percentage);
		setTooltipVisible(true, percentage);
	}

	function handleMove(e: MouseEvent | TouchEvent) {
		const percentage = getEventPercentage(e);
		onProgressMove?.(percentage);
		setTooltipVisible(true, percentage);
	}

	function handleLeave(e: MouseEvent | TouchEvent) {
		const percentage = getEventPercentage(e);
		onProgressLeave?.(percentage);
		setTooltipVisible(false);
	}

	// Mouse and touch wrappers
	const handleMouseEnter = (e: MouseEvent) => handleEnter(e);
	const handleMouseMove = (e: MouseEvent) => handleMove(e);
	const handleMouseLeave = (e: MouseEvent) => handleLeave(e);
	const handleTouchStart = (e: TouchEvent) => handleEnter(e);
	const handleTouchMove = (e: TouchEvent) => handleMove(e);
	const handleTouchEnd = () => {
		const percentage = (touchLastX / containerWidth) * 100;
		onProgressLeave?.(percentage);
		onProgressClick?.(percentage);
		setTooltipVisible(false);
	};

	function handleClick(e: MouseEvent) {
		const percentage = getEventPercentage(e);
		onProgressClick?.(percentage);
	}

	function updateContainerWidth() {
		if (container) {
			containerWidth = container.offsetWidth;
		}
	}

	function getProgressHeight() {
		switch (size) {
			case 'sm':
				return 3;
			case 'md':
				return 4;
			case 'lg':
				return 5;
		}
	}

	function getHandlerHeight() {
		switch (size) {
			case 'sm':
				return 16;
			case 'md':
				return 28;
			case 'lg':
				return 36;
		}
	}

	$effect(() => {
		updateContainerWidth();
	});
</script>

<svelte:window onresize={updateContainerWidth} />

<div class="relative {className}" bind:this={container}>
	{#if showTooltip}
		<div
			class="animate__animated animate__faster absolute top-[-2.5rem] w-fit rounded-lg border px-2 py-1 text-sm shadow-xl
				{tooltipVisible ? 'animate__fadeIn' : 'animate__fadeOut'}"
			style:left="{tooltipPosition}px"
			bind:this={tooltip}
		>
			{tooltipText}
		</div>
	{/if}

	<div
		class="absolute left-0 z-10 w-full cursor-pointer"
		style="
            bottom: -{getHandlerHeight() / 2}px;
            height: {getHandlerHeight()}px;
         "
		onmouseenter={handleMouseEnter}
		onmousemove={handleMouseMove}
		onmouseleave={handleMouseLeave}
		ontouchstart={handleTouchStart}
		ontouchmove={handleTouchMove}
		ontouchend={handleTouchEnd}
		onclick={handleClick}
	></div>

	<div
		class="pointer-events-none absolute top-1/2 w-full -translate-y-1/2 rounded-full bg-white opacity-30"
		style="height: {getProgressHeight()}px;"
	></div>

	<div
		class="pointer-events-none absolute left-0 top-1/2 -translate-y-1/2 rounded-full bg-white transition-all duration-200 ease-linear"
		style="height: {getProgressHeight()}px; width: {progressPercentage}%;"
	></div>

	<input
		class="absolute top-1/2 w-full -translate-y-1/2 cursor-pointer opacity-0"
		type="range"
		bind:value
		{min}
		{max}
		{step}
		onchange={() => onValueChange?.(value)}
		style="height: {getProgressHeight()}px; z-index: 5;"
	/>
</div>
