// Self-check for `updatekey.ts`: the UI asks for updates only once the app carries a signing key.
// `updatekey-conf.check.mjs` checks that key against the one `src-tauri/tauri.conf.json` ships.
//
//     node --experimental-strip-types ui/src/lib/updatekey.check.ts
//
// The bug this guards: an updater with no key answers every check with a signature error, which
// the Settings check would show as a failure on every launch.
import { updatesConfigured } from './updatekey.ts';

function ok(cond: boolean, what: string): void {
	if (!cond) throw new Error(`FAIL: ${what}`);
}

ok(!updatesConfigured(''), 'no key: no update checks');
ok(!updatesConfigured('   '), 'a blank key counts as none');
ok(updatesConfigured('dW50cnVzdGVkIGNvbW1lbnQ6'), 'a key: checks run');
console.log('ok');
