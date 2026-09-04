<script lang="ts">
	import { isMacos } from '$lib/platform';
	import TauriDeveloperAPI from '$lib/tauri/TauriDeveloperAPI';
	import { onDestroy, onMount } from 'svelte';

	let panelRef: HTMLDivElement | undefined = $state();
	let isDragging = $state(false);
	let position = $state({ x: 20, y: 360 });
	let dragStart = { x: 0, y: 0 };
	let isVisible = $state(true);

	let totalRamBytes = $state(0);
	let totalAppRamBytes = $state(0);
	let totalAppWorkingSetBytes = $state(0);
	let totalAppPrivateWsBytes = $state(0);
	let totalAppCpuPercent = $state(0);
	let totalAppGpuPercent = $state(0);
	let memoryMode = $state<'private_ws' | 'working_set' | 'commit'>('private_ws');
	let processes = $state<
		{
			pid: number;
			name: string;
			is_main: boolean;
			ram_bytes: number;
			working_set_bytes: number;
			private_ws_bytes: number;
			cpu_percent: number;
			gpu_percent: number;
		}[]
	>([]);

	let pollInterval: ReturnType<typeof setInterval> | undefined;

	const formatBytes = (bytes: number) => {
		if (bytes === 0) return '0 MB';
		const mb = bytes / (1024 * 1024);
		if (mb >= 1024) {
			return `${(mb / 1024).toFixed(2)} GB`;
		}
		return `${mb.toFixed(1)} MB`;
	};

	const fetchMetrics = async () => {
		try {
			const metrics = await TauriDeveloperAPI.getDeveloperMetrics();
			totalRamBytes = metrics.total_ram_bytes;
			totalAppRamBytes = metrics.total_app_ram_bytes;
			totalAppWorkingSetBytes = metrics.total_app_working_set_bytes;
			totalAppPrivateWsBytes = metrics.total_app_private_ws_bytes;
			totalAppCpuPercent = metrics.total_app_cpu_percent;
			totalAppGpuPercent = metrics.total_app_gpu_percent;
			processes = metrics.processes;
		} catch (e) {
			console.error('Failed to fetch metrics', e);
		}
	};

	const getActiveTotalRam = () => {
		if (memoryMode === 'private_ws') return totalAppPrivateWsBytes;
		if (memoryMode === 'working_set') return totalAppWorkingSetBytes;
		return totalAppRamBytes;
	};

	const getProcessRam = (proc: {
		ram_bytes: number;
		working_set_bytes: number;
		private_ws_bytes: number;
	}) => {
		if (memoryMode === 'private_ws') return proc.private_ws_bytes;
		if (memoryMode === 'working_set') return proc.working_set_bytes;
		return proc.ram_bytes;
	};

	const toggleMemoryMode = () => {
		if (memoryMode === 'private_ws') memoryMode = 'working_set';
		else if (memoryMode === 'working_set') memoryMode = 'commit';
		else memoryMode = 'private_ws';
	};

	const getModeLabel = () => {
		if (memoryMode === 'private_ws') return 'Private WS';
		if (memoryMode === 'working_set') return 'Working Set';
		return 'Commit';
	};

	onMount(() => {
		fetchMetrics();
		pollInterval = setInterval(fetchMetrics, 1000);
	});

	onDestroy(() => {
		if (pollInterval) clearInterval(pollInterval);
	});

	const onPointerDown = (e: PointerEvent) => {
		isDragging = true;
		dragStart = { x: e.clientX - position.x, y: e.clientY - position.y };
		if (panelRef) {
			panelRef.setPointerCapture(e.pointerId);
		}
	};

	const onPointerMove = (e: PointerEvent) => {
		if (!isDragging) return;
		position = {
			x: e.clientX - dragStart.x,
			y: e.clientY - dragStart.y
		};
	};

	const onPointerUp = (e: PointerEvent) => {
		isDragging = false;
		if (panelRef) {
			panelRef.releasePointerCapture(e.pointerId);
		}
	};
</script>

{#if isVisible}
	<div
		bind:this={panelRef}
		class="fixed z-[100] flex max-h-96 w-96 flex-col border border-white bg-black text-white"
		style="left: {position.x}px; top: {position.y}px;"
		onpointerdown={onPointerDown}
		onpointermove={onPointerMove}
		onpointerup={onPointerUp}
		onpointercancel={onPointerUp}
	>
		<div
			class="flex cursor-move select-none items-center justify-between border-b border-white px-3 py-2 text-xs font-bold"
			onpointerdown={onPointerDown}
		>
			<span>Performance ({processes.length} procs)</span>
			{#if !isMacos()}
				<div class="flex items-center gap-1.5" onpointerdown={(e) => e.stopPropagation()}>
					<button
						class="border border-white px-2 py-0.5 text-[10px] uppercase tracking-wider hover:bg-white hover:text-black"
						title="Toggle mode: Private WS (Task Manager physical RAM), Working Set (Total physical RAM), Commit (Virtual RAM)"
						onclick={toggleMemoryMode}
					>
						Mode: {getModeLabel()}
					</button>
				</div>
			{/if}
		</div>

		<div
			class="flex flex-col gap-2 overflow-y-auto p-3 font-mono text-xs"
			onpointerdown={(e) => e.stopPropagation()}
		>
			<!-- Summary -->
			<div class="flex items-center justify-between">
				<span class="text-zinc-400">Total RAM{isMacos() ? '' : ` (${getModeLabel()})`}:</span>
				<span class="font-bold">
					{formatBytes(getActiveTotalRam())}
					{#if totalRamBytes > 0}
						<span class="text-[10px] text-zinc-500">
							({((getActiveTotalRam() / totalRamBytes) * 100).toFixed(1)}% of sys)
						</span>
					{/if}
				</span>
			</div>
			<div class="h-1.5 w-full bg-zinc-800">
				<div
					class="h-full bg-white transition-all duration-300"
					style="width: {totalRamBytes > 0 ? Math.min(100, (getActiveTotalRam() / totalRamBytes) * 100) : 0}%"
				></div>
			</div>

			<div class="mt-1 flex items-center justify-between">
				<span class="text-zinc-400">Total CPU:</span>
				<span class="font-bold">{totalAppCpuPercent.toFixed(1)}%</span>
			</div>
			<div class="h-1.5 w-full bg-zinc-800">
				<div
					class="h-full bg-white transition-all duration-300"
					style="width: {Math.min(100, Math.max(0, totalAppCpuPercent))}%"
				></div>
			</div>

			<div class="mt-1 flex items-center justify-between">
				<span class="text-zinc-400">Total GPU:</span>
				<span class="font-bold">{totalAppGpuPercent.toFixed(1)}%</span>
			</div>
			<div class="h-1.5 w-full bg-zinc-800">
				<div
					class="h-full bg-white transition-all duration-300"
					style="width: {Math.min(100, Math.max(0, totalAppGpuPercent))}%"
				></div>
			</div>

			<!-- Process Breakdown -->
			<div class="mt-3 border-t border-zinc-700 pt-2 text-[11px]">
				<div class="mb-1 flex justify-between font-bold text-zinc-400">
					<span>Process</span>
					<span class="flex gap-2">
						<span class="w-14 text-right">RAM</span>
						<span class="w-10 text-right">CPU</span>
						<span class="w-10 text-right">GPU</span>
					</span>
				</div>
				<div class="flex flex-col gap-1">
					{#each processes as proc (proc.pid)}
						<div
							class="flex items-center justify-between border-b border-zinc-900 pb-1 text-[10px]"
							class:text-white={proc.is_main}
							class:text-zinc-300={!proc.is_main}
						>
							<span class="max-w-[140px] truncate" title={`${proc.name} (PID: ${proc.pid})`}>
								{proc.is_main ? '★ ' : '  '}{proc.name}
								<span class="text-zinc-500">({proc.pid})</span>
							</span>
							<span class="flex gap-2">
								<span class="w-14 text-right font-medium">
									{formatBytes(getProcessRam(proc))}
								</span>
								<span class="w-10 text-right font-medium">{proc.cpu_percent.toFixed(1)}%</span>
								<span class="w-10 text-right font-medium">{proc.gpu_percent.toFixed(1)}%</span>
							</span>
						</div>
					{/each}
				</div>
			</div>
		</div>
	</div>
{/if}
