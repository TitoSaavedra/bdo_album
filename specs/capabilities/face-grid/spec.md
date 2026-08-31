# Capability: face-grid

> LIVING truth — keep this current. Changes arrive as `specs/changes/<name>/spec-delta.md`
> and are merged here when the change is archived. This file should always describe what the
> system does *now*, not how it got here.

## Purpose
Lets the user apply a downloaded preset (or a saved combination of `FaceTexture` slots)
onto their own local Black Desert Online character accounts, outside of the
scraped-presets browsing flow.

## Requirements
- Reads the local BDO installation to discover accounts and their existing
  `FaceTexture`/`UserCache` character face images.
- A "face grid" is a saved arrangement of character face slots the user can name, apply
  back onto an account (overwriting `FaceTexture` images — a destructive, confirmed
  action), or delete.
- Applying or deleting a grid requires explicit confirmation in the UI (irreversible
  local file overwrite).
- Missing `UserCache`/`FaceTexture` directories are handled gracefully rather than
  crashing (the game may not have been run yet, or an account may be fresh).

## Behavior / scenarios
- **Given** BDO is installed and has been logged into at least once **when** the user
  opens the Grid tab **then** their local accounts and current face textures are listed.
- **Given** the user picks slots into a grid and saves it **when** they later apply that
  grid to an account **then** the confirm dialog warns that `FaceTexture` images will be
  overwritten before proceeding.
- **Given** no BDO accounts are found **when** the user opens the Grid tab **then** a
  hint explains BDO needs to be installed and logged in at least once.

## Notes
- Fully local/offline — does not touch the shared Postgres DB or R2; it's a separate
  concern from `preset-scraping`/`album-browsing` that happens to live in the same
  `album/` Tauri app (second top-level tab).
- Documents-folder resolution goes through `dirs::document_dir()` and tolerates a missing
  `UserCache`/`FaceTexture` dir rather than failing.
