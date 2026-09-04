import axios from 'axios';
import { createWriteStream } from 'node:fs';
import { pipeline } from 'node:stream/promises';
import fs from 'fs/promises';
import path from 'node:path';
import Seven from 'node-7z';
import extract from 'extract-zip';
import * as tar from 'tar';

export async function downloadFile(url: string, destPath: string): Promise<void> {
	console.log('Downloading', url);
	const response = await axios.get(url, { responseType: 'stream' });
	await pipeline(response.data, createWriteStream(destPath));
}

export async function extract7z(filePath: string, destination: string): Promise<void> {
	console.log('Extracting', filePath);
	await new Promise<void>((resolve, reject) => {
		const extractor = Seven.extractFull(filePath, destination, {
			$bin: undefined, // let node-7z find binary
			recursive: true,
			overwrite: 'a'
		});

		extractor.on('end', () => resolve());
		extractor.on('error', (err: any) => reject(err));
	});

	await fs.rm(filePath);
}

export async function extractZip(filePath: string, destination: string): Promise<void> {
	console.log('Extracting', filePath);
	await extract(filePath, { dir: path.resolve(destination) });
	await fs.rm(filePath);
}

export async function extractTarGz(filePath: string, destination: string): Promise<void> {
	console.log('Extracting', filePath);
	await tar.extract({
		file: filePath,
		cwd: destination
	});
	await fs.rm(filePath);
}
