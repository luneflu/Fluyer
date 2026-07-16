import * as path from 'path';
import fs from 'fs/promises';
import { downloadFile } from './install-helpers';
import { installBass } from './install-bass';

// Parse CLI arguments
const VALID_ARCHS = ['arm64-v8a', 'armeabi-v7a', 'x86', 'x86_64'] as const;
type Arch = (typeof VALID_ARCHS)[number];

// Map Tauri/Rust arch names to Android NDK names
const ARCH_ALIASES: Record<string, Arch> = {
	aarch64: 'arm64-v8a',
	armv7: 'armeabi-v7a',
	i686: 'x86',
	// Direct mappings
	'arm64-v8a': 'arm64-v8a',
	'armeabi-v7a': 'armeabi-v7a',
	x86: 'x86',
	x86_64: 'x86_64'
};

function parseArgs(): Arch {
	const args = process.argv.slice(2);

	if (args.includes('--help') || args.includes('-h')) {
		console.log(`Usage: pnpm android:init [--arch <architecture>]

Options:
  --arch, -a    Target architecture (default: arm64-v8a)
                Valid values: ${VALID_ARCHS.join(', ')}
                Aliases: aarch64 -> arm64-v8a, armv7 -> armeabi-v7a, i686 -> x86
  --help, -h    Show this help message

Examples:
  pnpm android:init
  pnpm android:init --arch arm64-v8a
  pnpm android:init -a aarch64
  pnpm android:init -a x86_64
`);
		process.exit(0);
	}

	const archIndex = args.findIndex((arg) => arg === '--arch' || arg === '-a');
	if (archIndex !== -1 && args[archIndex + 1]) {
		const inputArch = args[archIndex + 1];
		const arch = ARCH_ALIASES[inputArch];
		if (!arch) {
			console.error(`Invalid architecture: ${inputArch}`);
			console.error(`Valid architectures: ${VALID_ARCHS.join(', ')}`);
			console.error(`Aliases: aarch64, armv7, i686`);
			process.exit(1);
		}
		return arch;
	}

	return 'arm64-v8a'; // default
}

const ARCH = parseArgs();
console.log(`Target architecture: ${ARCH}\n`);

const version = '6.0';
const filename = 'ffmpeg-kit-main.aar';
const destDir = path.resolve('src-tauri', 'tauri-plugin-fluyer', 'android', 'libs');
const ffmpegDestPath = path.resolve(destDir, filename);
const downloadUrl = `https://github.com/alvindimas05/ffmpeg-kit/releases/download/v${version}/${filename}`;

const bassDestPath = path.resolve('src-tauri', 'gen', 'android', 'app', 'src', 'main', 'jniLibs');

async function installFFmpeg() {
	try {
		await fs.access(ffmpegDestPath);
		console.log('FFmpeg for Android is already installed. Reinstalling...');
		await fs.rm(ffmpegDestPath);
	} catch {}
	try {
		console.log('Installing FFmpeg for Android...');
		await fs.mkdir(destDir, { recursive: true });
		await downloadFile(downloadUrl, ffmpegDestPath);
	} catch (err) {
		console.error('Failed:', err);
	}
}

async function runBASSInstall() {
	await installBass({
		platform: 'android',
		arch: ARCH,
		destDir: bassDestPath
	});
}

Promise.all([runBASSInstall(), installFFmpeg()]);
