import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import crypto from 'node:crypto';

const pkgName = 'fluyer';
const aurRepoUrl = 'ssh://aur@aur.archlinux.org/fluyer.git'; 
const aurDir = join(process.cwd(), 'aur-repo');
const version = JSON.parse(readFileSync('package.json', 'utf-8')).version;
const owner = 'luneflu';
const repo = 'Fluyer';

console.log(`Updating AUR to version ${version}`);

// 1. Clone or pull AUR repo
try {
  execSync(`git clone ${aurRepoUrl} ${aurDir}`, { stdio: 'inherit' });
} catch (e) {
  console.log('AUR repo exists, pulling...');
  execSync(`git pull`, { cwd: aurDir, stdio: 'inherit' });
}

async function update() {
  // 2. Fetch remote asset to calculate SHA256
  const debUrl = `https://github.com/${owner}/${repo}/releases/download/v${version}/Fluyer_${version}_amd64.deb`;
  console.log(`Fetching ${debUrl} to calculate SHA256...`);
  
  let sha256 = '';
  try {
    const response = await fetch(debUrl);
    if (!response.ok) throw new Error(`Failed to fetch: ${response.statusText}`);
    const buffer = await response.arrayBuffer();
    const hash = crypto.createHash('sha256');
    hash.update(Buffer.from(buffer));
    sha256 = hash.digest('hex');
    console.log(`Calculated SHA256: ${sha256}`);
  } catch (error) {
    console.error(`Error calculating hash. Does the release v${version} exist on GitHub yet?`);
    console.error(error);
    process.exit(1);
  }

  // 3. Read PKGBUILD and update version & sum
  const pkgbuildPath = join(aurDir, 'PKGBUILD');
  let pkgbuild = readFileSync(pkgbuildPath, 'utf-8');

  pkgbuild = pkgbuild.replace(/^pkgver=.*$/m, `pkgver=${version}`);
  pkgbuild = pkgbuild.replace(/^pkgrel=.*$/m, `pkgrel=1`);
  pkgbuild = pkgbuild.replace(/^sha256sums_x86_64=\('.*'\)$/m, `sha256sums_x86_64=('${sha256}')`);

  writeFileSync(pkgbuildPath, pkgbuild);

  // 4. Update .SRCINFO manually
  const srcinfoPath = join(aurDir, '.SRCINFO');
  let srcinfo = readFileSync(srcinfoPath, 'utf-8');

  srcinfo = srcinfo.replace(/pkgver = .*$/m, `pkgver = ${version}`);
  srcinfo = srcinfo.replace(/pkgrel = .*$/m, `pkgrel = 1`);
  
  // Need to replace the version in the source URL globally
  const oldUrlRegex = /source_x86_64 = https:\/\/github\.com\/.*\/releases\/download\/v(.*)\/Fluyer_.*_amd64\.deb/g;
  srcinfo = srcinfo.replace(oldUrlRegex, `source_x86_64 = https://github.com/${owner}/${repo}/releases/download/v${version}/Fluyer_${version}_amd64.deb`);
  
  srcinfo = srcinfo.replace(/sha256sums_x86_64 = .*$/m, `sha256sums_x86_64 = ${sha256}`);

  writeFileSync(srcinfoPath, srcinfo);

  // 5. Commit and push
  try {
    execSync('git add PKGBUILD .SRCINFO', { cwd: aurDir, stdio: 'inherit' });
    execSync(`git commit -m "chore: bump version to ${version}"`, { cwd: aurDir, stdio: 'inherit' });
    console.log('Pushing to AUR...');
    // execSync('git push', { cwd: aurDir, stdio: 'inherit' });
    console.log('Done!');
  } catch(e) {
      console.log("Nothing to commit or push failed.");
  }
}

update();
