// The public key Aura's updates are signed with, the same string as `plugins.updater.pubkey` in
// `src-tauri/tauri.conf.json` (`updatekey.check.ts` keeps the two equal). Empty until hck0n3
// generates the key pair: `pnpm dlx @tauri-apps/cli signer generate -w ~/.tauri/aura.key`, the
// public half goes in both places and the private half in the TAURI_SIGNING_PRIVATE_KEY secret.
export const UPDATE_PUBKEY = '';

/** Whether this build can check for updates at all: without a key every answer fails its signature. */
export function updatesConfigured(key: string = UPDATE_PUBKEY): boolean {
	return key.trim().length > 0;
}
