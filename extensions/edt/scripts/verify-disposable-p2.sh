#!/bin/bash

set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: verify-disposable-p2.sh <java> <maven-repository> <work-root>" >&2
  exit 64
fi

java_executable=$1
maven_repository=$2
work_root=$3

repository_root=$(git rev-parse --show-toplevel)
edt_root="$repository_root/extensions/edt"
builder_root="$edt_root/tests/com.oneagent.edt.tests/target/work"
package_repository="$edt_root/repositories/com.oneagent.edt.repository/target/repository"
local_artifacts_root="$repository_root/local-artifacts"

canonical_existing() {
  local value=$1
  local directory
  directory=$(dirname "$value")
  printf '%s/%s\n' "$(cd "$directory" && pwd -P)" "$(basename "$value")"
}

java_executable=$(canonical_existing "$java_executable")
maven_repository=$(canonical_existing "$maven_repository")
builder_root=$(canonical_existing "$builder_root")
package_repository=$(canonical_existing "$package_repository")
local_artifacts_root=$(canonical_existing "$local_artifacts_root")

if [[ ! -x "$java_executable" ]]; then
  echo "java executable is missing or not executable" >&2
  exit 65
fi
if [[ ! -d "$maven_repository" || "$maven_repository" != "$local_artifacts_root"/* ]]; then
  echo "Maven repository must be under repository-local local-artifacts" >&2
  exit 65
fi
if [[ ! -d "$builder_root/configuration" || ! -d "$package_repository" ]]; then
  echo "run the EDT clean build before disposable p2 validation" >&2
  exit 65
fi

work_parent=$(canonical_existing "$(dirname "$work_root")")
work_root="$work_parent/$(basename "$work_root")"
if [[ "$work_root" != "$local_artifacts_root"/* || -e "$work_root" ]]; then
  echo "work root must be absent and under repository-local local-artifacts" >&2
  exit 65
fi

launcher_jar=$(find "$maven_repository" -type f \
  -path '*/org.eclipse.equinox.launcher/*/org.eclipse.equinox.launcher-*.jar' \
  -print | LC_ALL=C sort | tail -n 1)
if [[ -z "$launcher_jar" ]]; then
  echo "repository-local Equinox launcher is missing" >&2
  exit 65
fi

config_ini="$builder_root/configuration/config.ini"
if ! grep -Fq 'osgi.configuration.cascaded=false' "$config_ini" ||
   ! grep -Fq 'org.eclipse.equinox.p2.director.app' "$config_ini" ||
   ! grep -Fq 'org.eclipse.equinox.p2.transport.ecf' "$config_ini" ||
   ! grep -Fq 'org.eclipse.equinox.p2.touchpoint.eclipse' "$config_ini" ||
   ! grep -Fq 'org.eclipse.equinox.p2.touchpoint.natives' "$config_ini"; then
  echo "generated builder is not a standalone p2 director configuration" >&2
  exit 65
fi

mkdir -p "$work_root"
cp -R "$builder_root/configuration" "$work_root/builder-configuration"
cp -R "$builder_root/configuration" "$work_root/fresh-configuration"
mkdir -p \
  "$work_root/builder-data" \
  "$work_root/fresh-data" \
  "$work_root/user-home" \
  "$work_root/user-area" \
  "$work_root/p2-agent" \
  "$work_root/destination" \
  "$work_root/bundlepool"

profile=OneAgentSprint34Disposable
feature=com.oneagent.edt.feature.feature.group
repositories="file:$package_repository,https://download.eclipse.org/eclipse/updates/4.30/"

case "$(uname -s)" in
  Darwin)
    p2_os=macosx
    p2_ws=cocoa
    ;;
  Linux)
    p2_os=linux
    p2_ws=gtk
    ;;
  MINGW* | MSYS* | CYGWIN*)
    p2_os=win32
    p2_ws=win32
    ;;
  *)
    echo "unsupported p2 operating system" >&2
    exit 65
    ;;
esac

case "$(uname -m)" in
  arm64 | aarch64)
    p2_arch=aarch64
    ;;
  x86_64 | amd64)
    p2_arch=x86_64
    ;;
  *)
    echo "unsupported p2 architecture" >&2
    exit 65
    ;;
esac

director=(
  "$java_executable"
  "-Duser.home=$work_root/user-home"
  "-Declipse.p2.data.area=file:$work_root/p2-agent/"
  "-Dosgi.user.area=file:$work_root/user-area/"
  -jar "$launcher_jar"
  -nosplash
  -consoleLog
  -install "$builder_root"
  -configuration "$work_root/builder-configuration"
  -data "$work_root/builder-data"
  -application org.eclipse.equinox.p2.director
)

run_logged() {
  local name=$1
  shift
  "${director[@]}" "$@" 2>&1 | tee "$work_root/$name.log"
}

run_logged install \
  -repository "$repositories" \
  -installIU "$feature" \
  -destination "$work_root/destination" \
  -bundlepool "$work_root/bundlepool" \
  -profile "$profile" \
  -profileProperties org.eclipse.update.install.features=true \
  -p2.os "$p2_os" -p2.ws "$p2_ws" -p2.arch "$p2_arch" -roaming

run_logged list-installed \
  -listInstalledRoots \
  -destination "$work_root/destination" \
  -profile "$profile"
grep -Eq "^$feature/[^[:space:]]+$" "$work_root/list-installed.log"

run_logged uninstall \
  -uninstallIU "$feature" \
  -destination "$work_root/destination" \
  -profile "$profile"

director=(
  "$java_executable"
  "-Duser.home=$work_root/user-home"
  "-Declipse.p2.data.area=file:$work_root/p2-agent/"
  "-Dosgi.user.area=file:$work_root/user-area/"
  -jar "$launcher_jar"
  -nosplash
  -consoleLog
  -install "$builder_root"
  -configuration "$work_root/fresh-configuration"
  -data "$work_root/fresh-data"
  -application org.eclipse.equinox.p2.director
)

run_logged list-after-uninstall \
  -listInstalledRoots \
  -destination "$work_root/destination" \
  -profile "$profile"
if grep -Fq "$feature/" "$work_root/list-after-uninstall.log"; then
  echo "OneAgent root remains after uninstall" >&2
  exit 66
fi

profile_registry="$work_root/destination/p2/org.eclipse.equinox.p2.engine/profileRegistry/$profile.profile"
current_profile=$(find "$profile_registry" -type f -name '*.profile.gz' -print \
  | LC_ALL=C sort | tail -n 1)
if [[ -z "$current_profile" ]]; then
  echo "current disposable profile state is missing" >&2
  exit 66
fi
gzip -dc "$current_profile" >"$work_root/current-profile.xml"
if grep -Fq 'com.oneagent.edt' "$work_root/current-profile.xml"; then
  echo "OneAgent installable unit remains in disposable profile" >&2
  exit 66
fi

printf 'install=PASS\nlist=PASS\nuninstall=PASS\nfresh-list=PASS\nresult=PASS\n'
