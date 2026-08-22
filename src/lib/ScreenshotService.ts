import { snapdom } from '@zumer/snapdom';
import { invoke } from '@tauri-apps/api/core';
import { TauriCommands } from '$lib/constants/TauriCommands';

export class ScreenshotService {
	/**
	 * Takes a screenshot of the app's DOM after a specified delay and saves it to the home directory.
	 * @param delayMs Delay in milliseconds before taking the screenshot.
	 */
	static async takeScreenshot(delayMs: number = 10000) {
		console.log(`[ScreenshotService] Waiting for ${delayMs}ms before taking screenshot...`);

		await new Promise((resolve) => setTimeout(resolve, delayMs));

		try {
			console.log(`[ScreenshotService] Capturing DOM...`);
			const img = await snapdom.toPng(document.body, {
				backgroundColor: 'transparent'
			});

			const dataUrl = img.src;
			console.log(`[ScreenshotService] Processing and saving image...`);

			// Extract the base64 string from the data URL (format: "data:image/png;base64,...")
			const base64Data = dataUrl.includes(',') ? dataUrl.split(',')[1] : dataUrl;

			await invoke(TauriCommands.DEVELOPER_SCREENSHOT_SAVE, {
				base64Data: base64Data
			});
			console.log(`[ScreenshotService] Screenshot successfully saved.`);
		} catch (error) {
			console.error(`[ScreenshotService] Failed to take screenshot:`, error);
		}
	}
}
