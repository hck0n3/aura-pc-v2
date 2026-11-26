// The key the UI holds (`updatekey.ts`) is the one `src-tauri/tauri.conf.json` ships, and the
// updater asks hck0n3/aura-pc-v2, never the project Aura is based on.
//
//     node ui/src/lib/updatekey-conf.check.mjs
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const ok = (cond, what) => {
	if (!cond) throw new Error(`FAIL: ${what}`);
};

const ts = readFileSync(join(here, 'updatekey.ts'), 'utf8');
const ui = ts.match(/export const UPDATE_PUBKEY =\s*'([^']*)';/)?.[1];
ok(ui !== undefined, 'updatekey.ts declares UPDATE_PUBKEY as a plain string');

const conf = JSON.parse(readFileSync(join(here, '../../../src-tauri/tauri.conf.json'), 'utf8'));
const shipped = conf.plugins?.updater?.pubkey ?? '';
ok(ui === shipped, 'the UI copy of the key is the one tauri.conf.json ships');
ok(!shipped.includes('OTQwQ0MzMDU1QUYwMjQ1NA'), 'not the previous project signing key');
const endpoints = conf.plugins?.updater?.endpoints ?? [];
ok(endpoints.length > 0, 'an update endpoint is configured');
ok(
	endpoints.every((u) => u.startsWith('https://github.com/hck0n3/aura-pc-v2/')),
	'updates come from hck0n3/aura-pc-v2'
);
console.log('ok');
