#!/usr/bin/env bash
# Guardián de marca de Aura: falla mientras quede rastro de Discord, de los servicios y la
# infraestructura del proyecto del que Aura deriva, o de su nombre en archivos versionados.
#
# Uso: scripts/check-aura-brand.sh [discord|community|listen|infra|brand|all]
# Sale 0 si no hay coincidencias; 1 y la lista archivo:línea si las hay.
#
# Nunca se tocan (GPL-3.0 obliga a conservarlos): LICENSE, THIRD-PARTY-NOTICES.md y las líneas
# de copyright. La línea legal de "Acerca de" ("Basado en Limusic — GPL-3.0") está permitida.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

category="${1:-all}"

declare -A patterns=(
	[discord]='discord'
	[community]='lyrics\.limusic\.app|limusic_token|LIMUSIC_URL|limusic_(providers|status|sign_in|sign_out|me|rename|vote|report|review)'
	[listen]='sync\.limusic\.app|listentogether|listen[-_]protocol|sync-server|devicelink|LtSession|LinkSession'
	[infra]='SimoHypers|ko-fi|weblate|limusic\.app'
	[brand]='limusic'
)

excludes=(
	':(exclude)LICENSE'
	':(exclude)THIRD-PARTY-NOTICES.md'
	':(exclude)docs/superpowers/**'
	':(exclude)Cargo.lock'
	':(exclude)ui/pnpm-lock.yaml'
	':(exclude)scripts/check-aura-brand.sh'
)

# Líneas permitidas aunque coincidan (solo para la categoría brand).
allow='Basado en Limusic — GPL-3\.0|Copyright'

run() {
	local name="$1" out
	out="$(git grep -nI -i -E "${patterns[$name]}" -- . "${excludes[@]}" || true)"
	if [[ "$name" == brand && -n "$out" ]]; then
		out="$(grep -v -E "$allow" <<<"$out" || true)"
	fi
	if [[ -n "$out" ]]; then
		echo "✗ $name: $(wc -l <<<"$out") coincidencias"
		echo "$out"
		return 1
	fi
	echo "✓ $name"
}

status=0
if [[ "$category" == all ]]; then
	for c in discord community listen infra brand; do run "$c" || status=1; done
elif [[ -n "${patterns[$category]:-}" ]]; then
	run "$category" || status=1
else
	echo "categoría desconocida: $category" >&2
	exit 2
fi
exit "$status"
