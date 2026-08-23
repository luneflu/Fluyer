<script lang="ts">
    import { isDesktop, isAndroid } from '$lib/platform';
    import View from '$lib/ui/components/View.svelte';
    import { logs } from '$lib/services/LogService.svelte';
    import { tick } from 'svelte';
    import { writeText } from '@tauri-apps/plugin-clipboard-manager';

    let panelRef: HTMLDivElement | undefined = $state();
    let logsContainerRef: HTMLDivElement | undefined = $state();
    let isDragging = $state(false);
    let position = $state({ x: 20, y: 20 });
    let dragStart = { x: 0, y: 0 };
    let autoScroll = $state(true);
    let isVisible = $state(true);

    $effect(() => {
        if (autoScroll && logsContainerRef && logs.length > 0) {
            logsContainerRef.scrollTop = logsContainerRef.scrollHeight;
        }
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

    const clearLogs = () => {
        logs.length = 0;
    };
    
    const copyLogs = async () => {
        const logText = logs.map(l => `[${l.level}] ${l.message}`).join('\n');
        await writeText(logText);
    };
    
    const onScroll = () => {
        if (!logsContainerRef) return;
        const isAtBottom = logsContainerRef.scrollHeight - logsContainerRef.scrollTop <= logsContainerRef.clientHeight + 10;
        autoScroll = isAtBottom;
    };
</script>

{#if isVisible}
<div
    bind:this={panelRef}
    class="fixed z-[100] flex flex-col w-96 h-80 bg-black text-white border border-white"
    style="left: {position.x}px; top: {position.y}px;"
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointercancel={onPointerUp}
>
    <div class="px-3 py-2 border-b border-white font-bold text-xs select-none cursor-move flex items-center justify-between" onpointerdown={onPointerDown}>
        <span>Logs</span>
        <div class="flex gap-2">
            <button class="border border-white px-2 py-1 hover:bg-white hover:text-black" onclick={copyLogs}>Copy</button>
            <button class="border border-white px-2 py-1 hover:bg-white hover:text-black" onclick={() => autoScroll = !autoScroll}>
                {autoScroll ? 'Auto-scroll On' : 'Auto-scroll Off'}
            </button>
            <button class="border border-white px-2 py-1 hover:bg-white hover:text-black" onclick={clearLogs}>Clear</button>
            <button class="border border-white px-2 py-1 hover:bg-white hover:text-black" onclick={() => isVisible = false}>X</button>
        </div>
    </div>
    
    <div 
        bind:this={logsContainerRef}
        class="flex-1 p-2 overflow-y-auto font-mono text-[10px] space-y-1"
        onpointerdown={(e) => e.stopPropagation()} 
        onscroll={onScroll}
    >
        {#each logs as log}
            <div class="break-all whitespace-pre-wrap">
                <span>[{log.level}]</span> {log.message}
            </div>
        {/each}
    </div>
</div>
{/if}
