import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, '..');

// ─── helpers ────────────────────────────────────────────────────────────────

function parseEnv(content: string): Record<string, string> {
	const map: Record<string, string> = {};
	for (const line of content.split('\n')) {
		const trimmed = line.trim();
		if (!trimmed || trimmed.startsWith('#')) continue;
		const idx = trimmed.indexOf('=');
		if (idx === -1) continue;
		const key = trimmed.slice(0, idx).trim();
		const val = trimmed
			.slice(idx + 1)
			.trim()
			.replace(/^["']|["']$/g, '');
		map[key] = val;
	}
	return map;
}

/** Compute a monotonically-increasing integer versionCode from semver.
 *  Strategy: major * 1_000_000 + minor * 1_000 + patch
 *  e.g. 1.2.0 → 1_002_000
 */
function toVersionCode(semver: string): number {
	const [major = 0, minor = 0, patch = 0] = semver.split('.').map(Number);
	return major * 1_000_000 + minor * 1_000 + patch;
}

// ─── updaters ────────────────────────────────────────────────────────────────

async function syncPackageJson(version: string) {
	const file = path.join(ROOT, 'package.json');
	const json = JSON.parse(await fs.readFile(file, 'utf8'));
	if (json.version === version) {
		console.log(`  package.json          already ${version}`);
		return;
	}
	json.version = version;
	await fs.writeFile(file, JSON.stringify(json, null, '\t') + '\n');
	console.log(`  package.json          → ${version}`);
}

async function syncCargoToml(version: string) {
	const file = path.join(ROOT, 'src-tauri', 'Cargo.toml');
	const content = await fs.readFile(file, 'utf8');
	// Only replace the version inside [package] section
	const updated = content.replace(/^(version\s*=\s*)"[^"]*"/m, `$1"${version}"`);
	if (updated === content) {
		console.log(`  Cargo.toml            already ${version}`);
		return;
	}
	await fs.writeFile(file, updated);
	console.log(`  Cargo.toml            → ${version}`);
}

async function syncTauriConf(version: string) {
	const file = path.join(ROOT, 'src-tauri', 'tauri.conf.json');
	const json = JSON.parse(await fs.readFile(file, 'utf8'));
	if (json.version === version) {
		console.log(`  tauri.conf.json       already ${version}`);
		return;
	}
	json.version = version;
	await fs.writeFile(file, JSON.stringify(json, null, '\t') + '\n');
	console.log(`  tauri.conf.json       → ${version}`);
}

async function syncEnvViteVersion(version: string) {
	const file = path.join(ROOT, '.env');
	const content = await fs.readFile(file, 'utf8');
	const updated = content.replace(/^(VITE_APP_VERSION\s*=\s*).*/m, `$1${version}`);
	if (updated === content) {
		// VITE_APP_VERSION line might not exist, append it
		if (!content.includes('VITE_APP_VERSION')) {
			await fs.appendFile(file, `\nVITE_APP_VERSION=${version}\n`);
			console.log(`  .env VITE_APP_VERSION  → ${version} (appended)`);
		} else {
			console.log(`  .env VITE_APP_VERSION  already ${version}`);
		}
		return;
	}
	await fs.writeFile(file, updated);
	console.log(`  .env VITE_APP_VERSION  → ${version}`);
}

async function syncAndroidTauriProperties(version: string) {
	const file = path.join(ROOT, 'src-tauri', 'gen', 'android', 'app', 'tauri.properties');
	const versionCode = toVersionCode(version);
	const content =
		`tauri.android.versionCode=${versionCode}\n` + `tauri.android.versionName=${version}\n`;

	try {
		const existing = await fs.readFile(file, 'utf8');
		if (existing === content) {
			console.log(`  tauri.properties      already ${version} (code ${versionCode})`);
			return;
		}
	} catch {
		// file doesn't exist yet — will be created
	}

	await fs.writeFile(file, content);
	console.log(`  tauri.properties      → ${version} (code ${versionCode})`);
}

// ─── main ────────────────────────────────────────────────────────────────────

async function main() {
	const envPath = path.join(ROOT, '.env');
	const envContent = await fs.readFile(envPath, 'utf8');
	const env = parseEnv(envContent);

	const version = env['APP_VERSION'];
	if (!version) {
		console.error('Error: APP_VERSION not found in .env');
		process.exit(1);
	}

	// Basic semver validation
	if (!/^\d+\.\d+\.\d+/.test(version)) {
		console.error(`Error: APP_VERSION "${version}" doesn't look like a semver (x.y.z)`);
		process.exit(1);
	}

	console.log(`\nSyncing version ${version} from .env …\n`);

	await Promise.all([
		syncPackageJson(version),
		syncCargoToml(version),
		syncTauriConf(version),
		syncEnvViteVersion(version),
		syncAndroidTauriProperties(version)
	]);

	console.log('\nDone! ✓');
}

main().catch((err) => {
	console.error(err);
	process.exit(1);
});
