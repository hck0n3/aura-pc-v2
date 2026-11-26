// The public key Aura's updates are signed with, the same string as `plugins.updater.pubkey` in
// `src-tauri/tauri.conf.json` (`updatekey-conf.check.mjs` keeps the two equal). hck0n3's key,
// made with `@tauri-apps/cli signer generate`; its private half is the TAURI_SIGNING_PRIVATE_KEY
// secret and never enters the repo. Empty would switch update checks off.
export const UPDATE_PUBKEY =
	'dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDZCNjU3MjQ0QUNGMUVGRDkKUldUWjcvR3NSSEpsYTB3R0plR2JJc1MvVW5vYURZdEJlbHBZZDIzaHUzSmc4MHI3WmlBdlNETEIK';

/** Whether this build can check for updates at all: without a key every answer fails its signature. */
export function updatesConfigured(key: string = UPDATE_PUBKEY): boolean {
	return key.trim().length > 0;
}
