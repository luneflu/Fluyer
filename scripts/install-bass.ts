import path from 'path';
import { downloadFile, extractZip } from './install-helpers';
import * as fs from 'fs/promises';
import * as os from 'os';

const VERSION = 24;
const DEFAULT_LIBS = [
	'bass',
	'bassmix',
	'bassflac',
	'bassopus',
	'bassape',
	'bassalac',
	'basswv',
	'bass_aac'
];

export interface InstallOptions {
	platform?: 'android' | NodeJS.Platform;
	arch?: string;
	destDir?: string;
}

export async function installBassLib(name: string, options: InstallOptions = {}) {
	const platform = options.platform || os.platform();
	const arch = options.arch || process.env.ARCH || process.env.arch || os.arch();
	const destPath = options.destDir || path.resolve('src-tauri', 'libs');

	if (platform === 'darwin' && (name === 'bassalac' || name === 'bass_aac')) {
		console.log(`Skipping ${name} for macOS...`);
		return;
	}

	const downloadPath = path.join(destPath, `${name}-${VERSION}-${platform}-${arch}.zip`);
	const extractPath = path.join(destPath, `${name}-${VERSION}-${platform}-${arch}`);

	// Ensure destination directory exists (for downloading zip)
	await fs.mkdir(destPath, { recursive: true });

	let downloadUrl = '';
	let libSourcePath = '';
	let libDestPath = '';
	let extraOps: (() => Promise<void>) | null = null;

	try {
		if (platform === 'android') {
			if (!options.arch) throw new Error('Arch is required for Android installation');

			downloadUrl = `https://www.un4seen.com/files/${name === 'bass_aac' ? 'z/2/' : ''}${name}${VERSION}-android.zip`;
			// Android zip structure: libs/<arch>/lib<name>.so
			libSourcePath = path.join(extractPath, 'libs', arch, `lib${name}.so`);

			// Destination: <destDir>/<arch>/lib<name>.so
			const archDir = path.join(destPath, arch);
			await fs.mkdir(archDir, { recursive: true });
			libDestPath = path.join(archDir, path.basename(libSourcePath));
		} else {
			let platformSuffix = '';
			switch (platform) {
				case 'win32':
					platformSuffix = '';
					libSourcePath = path.join(extractPath, 'x64', `${name}.dll`);
					// Some bass zip files put dll in root directory instead of x64
					try {
						await fs.access(libSourcePath);
					} catch {
						libSourcePath = path.join(extractPath, `${name}.dll`);
					}
					libDestPath = path.join(destPath, `${name}.dll`);

					extraOps = async () => {
						let libWindowsPath = path.join(extractPath, 'c', 'x64', `${name}.lib`);
						try {
							await fs.access(libWindowsPath);
						} catch {
							libWindowsPath = path.join(extractPath, 'c', `${name}.lib`);
						}
						const destLibWindowsPath = path.join(destPath, path.basename(libWindowsPath));
						try {
							await fs.copyFile(libWindowsPath, destLibWindowsPath);
						} catch (e) {}
					};
					break;
				case 'darwin':
					platformSuffix = '-osx';
					libSourcePath = path.join(extractPath, `lib${name}.dylib`);
					libDestPath = path.join(destPath, `lib${name}.dylib`);
					break;
				case 'linux':
					platformSuffix = '-linux';
					let archFolder = '';
					// Use provided arch or fallback to os.arch() logic if it matches expected input
					const currentArch = options.arch || os.arch();
					switch (currentArch) {
						case 'x64':
							archFolder = 'x86_64';
							break;
						case 'ia32':
							archFolder = 'x86';
							break;
						case 'arm':
							archFolder = 'armhf';
							break;
						case 'arm64':
							archFolder = 'aarch64';
							break;
						default:
							throw new Error(`Unsupported architecture: ${currentArch}`);
					}
					libSourcePath = path.join(extractPath, 'libs', archFolder, `lib${name}.so`);
					libDestPath = path.join(destPath, `lib${name}.so`);
					break;
				default:
					throw new Error(`Unsupported platform: ${platform}`);
			}
			if (!downloadUrl) {
				downloadUrl = `https://www.un4seen.com/files/${name === 'bass_aac' ? 'z/2/' : ''}${name}${VERSION}${platformSuffix}.zip`;
			}
		}

		try {
			await fs.access(libDestPath);
			console.log(`${name} is already installed. Reinstalling...`);
			await fs.rm(libDestPath);
		} catch (e) {}

		console.log(
			`Installing ${name} for ${platform}${platform === 'android' ? ` (${arch})` : ''}...`
		);

		let attempts = 0;
		const maxAttempts = 3;
		while (attempts < maxAttempts) {
			try {
				await downloadFile(downloadUrl, downloadPath);
				await extractZip(downloadPath, extractPath);

				if (platform === 'win32') {
					libSourcePath = path.join(extractPath, 'x64', `${name}.dll`);
					try {
						await fs.access(libSourcePath);
					} catch {
						libSourcePath = path.join(extractPath, `${name}.dll`);
					}
				}

				await fs.copyFile(libSourcePath, libDestPath);

				if (extraOps) {
					await extraOps();
				}
				console.log(`Successfully installed ${name} to ${libDestPath}`);
				break;
			} catch (error) {
				attempts++;
				console.error(
					`Attempt ${attempts} failed for ${name}:`,
					error instanceof Error ? error.message : String(error)
				);

				try {
					await fs.rm(downloadPath);
				} catch (e) {}
				try {
					await fs.rm(extractPath, { recursive: true, force: true });
				} catch (e) {}

				if (attempts >= maxAttempts) {
					throw error;
				}

				console.log(`Retrying ${name} in 1 second...`);
				await new Promise((resolve) => setTimeout(resolve, 1000));
			}
		}
	} finally {
		try {
			await fs.rm(downloadPath);
		} catch (e) {}
		try {
			await fs.rm(extractPath, { recursive: true, force: true });
		} catch (e) {}
	}
}

export async function installBass(options: InstallOptions = {}) {
	const platform = options.platform || os.platform();
	const arch = options.arch || process.env.ARCH || process.env.arch || os.arch();
	const destDir = options.destDir || path.resolve('src-tauri', 'libs');
	await fs.mkdir(destDir, { recursive: true });

	if (platform === 'win32' && (arch === 'arm64' || arch === 'aarch64')) {
		const downloadPath = path.join(destDir, `bass24-arm64.zip`);
		const extractPath = path.join(destDir, `bass24-arm64`);
		try {
			await downloadFile(`https://www.un4seen.com/files/bass${VERSION}-arm64.zip`, downloadPath);
			await extractZip(downloadPath, extractPath);

			for (const lib of DEFAULT_LIBS) {
				if (lib === 'bassalac' || lib === 'bass_aac') continue;
				const dllSrc = path.join(extractPath, 'arm64', `${lib}.dll`);
				const libSrc = path.join(extractPath, 'c', 'arm64', `${lib}.lib`);
				await fs.copyFile(dllSrc, path.join(destDir, `${lib}.dll`));
				await fs.copyFile(libSrc, path.join(destDir, `${lib}.lib`));
			}
		} finally {
			try {
				await fs.rm(extractPath, { recursive: true, force: true });
			} catch (e) {}
		}
		return;
	}

	const promises: Promise<void>[] = [];
	for (const lib of DEFAULT_LIBS) {
		promises.push(installBassLib(lib, options));
	}
	await Promise.all(promises);
}

// @ts-ignore
if (import.meta.main) {
	installBass().catch(console.error);
}
