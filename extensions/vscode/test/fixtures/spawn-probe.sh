#!/bin/sh
set -eu
: > "${ONEAGENT_SPAWN_MARKER:?}"
exit 1
