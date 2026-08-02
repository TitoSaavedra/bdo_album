#!/usr/bin/env python3
"""Applies pending .sql migrations from scraper/src-tauri/migrations to Postgres.

Local dev replacement for `sqlx migrate run` — no Rust/sqlx-cli involved.
Tracks applied migrations in `_sqlx_migrations`, using the same table shape
and SHA-384 checksum algorithm sqlx itself uses, so it stays compatible with
rows already recorded there (and CI's own sqlx-cli-based migration step,
which is untouched by this script).

Usage:
    python scripts/migrate.py [--database-url URL] [--migrations-dir DIR]

DATABASE_URL is read from the environment, or from scraper/src-tauri/.env,
if --database-url isn't passed.

Requires: pip install psycopg2-binary
"""
import argparse
import hashlib
import os
import re
import sys
import time
from pathlib import Path

import psycopg2

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MIGRATIONS_DIR = REPO_ROOT / "scraper" / "src-tauri" / "migrations"
DEFAULT_ENV_FILE = REPO_ROOT / "scraper" / "src-tauri" / ".env"
ADVISORY_LOCK_KEY = "bdo_album_migrations"

MIGRATION_RE = re.compile(r"^(\d+)_(.+)\.sql$")


def load_database_url() -> str:
    url = os.environ.get("DATABASE_URL")
    if url:
        return url
    if DEFAULT_ENV_FILE.exists():
        for line in DEFAULT_ENV_FILE.read_text().splitlines():
            line = line.strip()
            if line.startswith("DATABASE_URL="):
                return line.split("=", 1)[1].strip().strip('"')
    sys.exit(f"DATABASE_URL not set (env var, or {DEFAULT_ENV_FILE})")


def discover_migrations(migrations_dir: Path):
    migrations = []
    for path in sorted(migrations_dir.glob("*.sql")):
        m = MIGRATION_RE.match(path.name)
        if not m:
            continue
        version = int(m.group(1))
        description = m.group(2).replace("_", " ")
        raw = path.read_bytes()
        checksum = hashlib.sha384(raw).digest()
        migrations.append((version, description, raw.decode("utf-8"), checksum, path.name))
    migrations.sort(key=lambda row: row[0])
    return migrations


def ensure_tracking_table(cur):
    cur.execute(
        """
        CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
            success BOOLEAN NOT NULL,
            checksum BYTEA NOT NULL,
            execution_time BIGINT NOT NULL
        )
        """
    )


def applied_migrations(cur):
    cur.execute("SELECT version, checksum FROM _sqlx_migrations")
    return {version: bytes(checksum) for version, checksum in cur.fetchall()}


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--database-url", default=None)
    parser.add_argument("--migrations-dir", default=str(DEFAULT_MIGRATIONS_DIR))
    args = parser.parse_args()

    database_url = args.database_url or load_database_url()
    migrations_dir = Path(args.migrations_dir)
    migrations = discover_migrations(migrations_dir)
    if not migrations:
        sys.exit(f"No migrations found in {migrations_dir}")

    conn = psycopg2.connect(database_url)
    conn.autocommit = False
    try:
        with conn.cursor() as cur:
            # Prevents two migrators (or two devs) from racing on the same DB.
            cur.execute("SELECT pg_advisory_lock(hashtext(%s))", (ADVISORY_LOCK_KEY,))
            ensure_tracking_table(cur)
        conn.commit()

        with conn.cursor() as cur:
            applied = applied_migrations(cur)

        applied_count = 0
        for version, description, sql, checksum, filename in migrations:
            if version in applied:
                if applied[version] != checksum:
                    sys.exit(
                        f"Checksum mismatch for migration {version} ({filename}) — "
                        f"the file's bytes changed after it was applied (e.g. line-ending "
                        f"drift from a git checkout). Never edit an already-applied "
                        f"migration's SQL; if this is just whitespace, restore the file's "
                        f"original bytes instead of re-running."
                    )
                continue

            print(f"Applying {filename}...")
            start = time.monotonic()
            with conn.cursor() as cur:
                cur.execute(sql)
                elapsed_ns = int((time.monotonic() - start) * 1_000_000_000)
                cur.execute(
                    "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) "
                    "VALUES (%s, %s, true, %s, %s)",
                    (version, description, psycopg2.Binary(checksum), elapsed_ns),
                )
            conn.commit()
            applied_count += 1
            print(f"  -> applied (version {version})")

        with conn.cursor() as cur:
            cur.execute("SELECT pg_advisory_unlock(hashtext(%s))", (ADVISORY_LOCK_KEY,))
        conn.commit()
    finally:
        conn.close()

    if applied_count == 0:
        print("Already up to date - nothing to apply.")
    else:
        print(f"Applied {applied_count} migration(s).")


if __name__ == "__main__":
    main()
