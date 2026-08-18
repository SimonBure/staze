#!/usr/bin/env bash
# Seeds a fresh sqlite db with sample sessions for demo.gif rendering.
# Dates are computed relative to "now" so the fixture never goes stale.
#
# Usage: seed_demo_db.sh <path-to-db>

set -euo pipefail

DB_PATH="$1"
NOW=$(date +%s)
DAY=86400

sqlite3 "$DB_PATH" <<'SQL'
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at INTEGER NOT NULL,
    duration_sec INTEGER NOT NULL,
    label TEXT
);
SQL

insert() {
    local days_ago="$1" duration_sec="$2" label="$3"
    local ts=$(( NOW - days_ago * DAY ))
    sqlite3 "$DB_PATH" "INSERT INTO sessions (started_at, duration_sec, label) VALUES ($ts, $duration_sec, '$label');"
}

# Last 9 days: daily sessions, including a short and a long outlier
# to exercise the bar-height floor/ceiling.
insert 0   10200  'coding'      # 2h50m
insert 1   26100  'coding'      # 7h15m
insert 2   1200   'meetings'    # 20m  (short outlier)
insert 3   24000  'writing'     # 6h40m
insert 4   37200  'coding'      # 10h20m (long outlier)
insert 5   14700  'coding'      # 4h05m
insert 6   2700   'meetings'    # 45m
insert 7   19800  'writing'     # 5h30m
insert 8   28800  'coding'      # 8h00m

# Sparser entries reaching back further, to populate month/year views.
insert 20  21600  'coding'      # 6h
insert 35  18000  'writing'     # 5h
insert 50  32400  'coding'      # 9h
insert 80  25200  'meetings'    # 7h
insert 110 14400  'writing'     # 4h
insert 150 21600  'coding'      # 6h
insert 200 18000  'coding'      # 5h
insert 250 10800  'writing'     # 3h
insert 300 28800  'coding'      # 8h
