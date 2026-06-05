# Reglas de ramas

## Ramas principales

| Rama | Uso |
|---|---|
| `main` | Producción. Siempre desplegable. Solo entra código vía PR. |
| `feat/*` | Nuevas funcionalidades |
| `fix/*` | Corrección de bugs |
| `chore/*` | CI, dependencias, docs, refactors sin cambio de comportamiento |
| `release/*` | Preparación de versión (opcional) |

## Flujo de trabajo

1. Crear rama desde `main` actualizada:
   ```bash
   git checkout main
   git pull origin main
   git checkout -b feat/mi-cambio
   ```
2. Commits pequeños y descriptivos (Conventional Commits: `feat:`, `fix:`, `chore:`…).
3. Abrir PR hacia `main`. Título: `[better-screenshoot] Descripción clara`.
4. Esperar CI verde (`frontend`, `rust`, `tauri-build`).
5. Merge (squash recomendado para historial limpio).

## Protección de `main`

Configura estas reglas en GitHub (Settings → Branches → Add rule):

- **Branch name pattern:** `main`
- Require a pull request before merging
- Require status checks to pass: `ci-success`
- Do not allow bypassing the above settings
- Restrict pushes (opcional): solo admins o nadie pushea directo

### Aplicar con script

Tras `gh auth login`:

```bash
./scripts/setup-branch-protection.sh
```

## Releases y versiones

- La versión vive en `Cargo.toml` (workspace), `package.json` raíz y `apps/desktop/src-tauri/tauri.conf.json`.
- Al preparar release, sincroniza la versión en todos esos archivos.
- Los tags siguen semver: `v0.2.0`, `v0.2.1`, `v1.0.0`.
- **Solo se etiqueta desde `main`**, después de mergear el PR de release.

```bash
git checkout main
git pull origin main
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions publicará un borrador de release con los instaladores (.dmg, .exe, .deb…).

## Qué no hacer

- No pushear directamente a `main` (salvo hotfixes acordados).
- No crear tags desde ramas de feature.
- No mezclar bump de versión con cambios de feature no relacionados.
