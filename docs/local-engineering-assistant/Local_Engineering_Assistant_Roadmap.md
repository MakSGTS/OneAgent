# Local Engineering Assistant Server Roadmap

## Purpose

Build a reliable local engineering AI server for software development, architecture analysis, documentation research, and private retrieval-augmented generation (RAG).

Primary workloads:

- llama.cpp with CUDA acceleration;
- an OpenAI-compatible local inference API;
- Open WebUI;
- OneAgent and engineering-domain knowledge bases;
- backup, monitoring, secure remote administration, and disaster recovery.

This file is the authoritative continuation point for the deployment. Every task must update the verified state, milestone status, last completed task, exact next task, and change history before it finishes.

## Status legend

- ✅ Completed — verified on the live server with recorded evidence.
- 🟡 In progress — the current bounded task.
- ⬜ Planned — not implemented or not yet verified.
- ⏸ Deferred — intentionally postponed.
- ❌ Abandoned — explicitly rejected and not part of the target architecture.
- ⚠ Requires verification — reported or historical state without current live evidence.

No milestone may be marked completed solely because an earlier handoff says it was completed.

## Deployment safety rules

1. Inspect before changing.
2. Record the exact target device, file, service, or container before acting.
3. Never format a disk identified only by a volatile name such as `/dev/sda` or `/dev/nvme0n1`; verify model, serial, size, and existing filesystems first.
4. Back up configuration and user data before structural changes.
5. Use `netplan try` for remote network changes.
6. Keep the previous bootable kernel available when changing the kernel or NVIDIA stack.
7. Do not remove models, Open WebUI data, vector data, or knowledge bases without explicit scope.
8. Do not run Certbot until DNS and the ACME HTTP path are stable.
9. Do not reintroduce Tailscale or WireGuard unless explicitly requested.
10. Perform one bounded milestone at a time and validate it before continuing.

## Verified target hardware

The following state was verified on the live host:

- ✅ Motherboard: ASRock X870E Nova WiFi; BIOS `4.43`, dated 2026-06-24.
- ✅ CPU: AMD Ryzen 7 9850X3D, 8 cores / 16 threads.
- ✅ GPU: ASUS-subsystem NVIDIA GeForce RTX 5090 at `0000:01:00.0`, PCI ID `10de:2b85`, with 32,607 MiB VRAM.
- ✅ RAM: 2 x 16 GB Kingston `KF580C38-16`, approximately 30 GiB usable.
- ✅ System disk: Samsung SSD 970 EVO Plus 250GB, serial `S4EUNF0M944981L`, stable ID `/dev/disk/by-id/nvme-eui.0025385991b26cf0`.
- ✅ RAG disk: 120GB SSD, serial `I38765R017695`, stable ID `/dev/disk/by-id/wwn-0x502b2a201d1c1b1a`.
- ✅ Wired NIC: Realtek RTL8126 5 GbE, PCI ID `10ec:8126`, using `r8169`.
- ✅ Wi-Fi: MediaTek adapter `wlp9s0`; disabled and outside the production network architecture.

## Target architecture

```text
Browser
  |
  v
Nginx (HTTP initially; HTTPS only after DNS validation)
  |
  v
Open WebUI (Docker, host networking)
  |
  v
llama.cpp OpenAI-compatible API (127.0.0.1:8080)
  |
  v
RTX 5090 + production GGUF model

Open WebUI
  |
  +-- SQLite application data on protected persistent storage
  +-- vector database on protected persistent storage
  +-- read-only source corpus on the dedicated RAG filesystem
```

Target persistent paths:

```text
/opt/src/llama.cpp
/opt/open-webui/compose.yaml
/etc/llama-server/
/srv/llm/models/
/srv/llm/data/open-webui/
/srv/llm/rag/
/srv/llm/backup/
/srv/llm/benchmarks/
/srv/llm/monitoring/
/srv/llm/logs/
/srv/llm/docs/
```

## Current milestone

🟡 **Milestone 6 — Service accounts and directory layout**

Milestone 5 driver and CUDA toolkit deliverables are complete and reboot-validated. Verification that the precompiled NVIDIA modules survive a future HWE kernel update remains a lifecycle check rather than a blocker for the next bounded task.

## Milestone 0 — Linux installation gate

### First mandatory question

> Is Linux installed, and what exact distribution and version is running?

If Linux is not installed, stop. The user installs Ubuntu Server 24.04 independently and returns after the first successful boot.

If Linux is installed, collect read-only evidence:

```bash
test -r /etc/os-release && cat /etc/os-release
uname -a
uname -m
hostnamectl
whoami
id
uptime
```

Acceptance criteria:

- ✅ Linux is installed and boots successfully.
- ✅ Distribution is Ubuntu Server 24.04.4 LTS.
- ✅ Architecture is `x86_64`.
- ✅ Current kernel is `7.0.0-30-generic` after enabling the Ubuntu HWE track.
- ✅ Local console access was used for protected network changes.

Stop conditions:

- unsupported or unexpected distribution;
- installation did not complete cleanly;
- filesystem or boot errors;
- no administrative access.

## Milestone 1 — Hardware and firmware inventory

Collect evidence before installing drivers or formatting disks:

```bash
lscpu
free -h
sudo dmidecode -t baseboard -t bios -t memory
lspci -nnk
lsblk -e7 -o NAME,PATH,SIZE,TYPE,FSTYPE,FSVER,LABEL,UUID,MOUNTPOINTS,MODEL,SERIAL
sudo blkid
findmnt
df -hT
swapon --show
mokutil --sb-state
```

Acceptance criteria:

- ✅ ASRock X870E Nova WiFi identity and BIOS `4.43` dated 2026-06-24 are verified.
- ✅ CPU is AMD Ryzen 7 9850X3D, 8 cores / 16 threads.
- ✅ Two 16 GB Kingston `KF580C38-16` modules and approximately 30 GiB usable RAM are confirmed.
- ✅ ASUS-subsystem NVIDIA GeForce RTX 5090 is visible at `0000:01:00.0`, PCI ID `10de:2b85`.
- ✅ RTL8126 `10ec:8126` is visible and bound to `r8169`.
- ✅ The system NVMe and RAG SATA SSD are distinguished by model, serial, and stable ID.
- ✅ The system LVM layout and dedicated RAG ext4 filesystem are documented.
- ✅ Secure Boot is enabled.

Firmware review:

- ✅ Onboard LAN is operational.
- ✅ The SATA controller and intended RAG SSD are operational.
- ⚠ Above 4G Decoding and Resizable BAR firmware settings remain unverified.
- ⚠ Both DIMMs are configured at 8000 MT/s; baseline memory stability is not yet proven.
- ⏸ BIOS update is deferred unless release-note review identifies a required fix and a rollback procedure is defined.

## Milestone 2 — Ethernet bootstrap and SSH

Production networking uses wired Ethernet. Wi-Fi may be used only as a temporary package-download path and must not become an undeclared production dependency.

Initial checks:

```bash
ip -br link
ip -br address
ip route
lspci -nnk -d 10ec:8126
modinfo r8169 | grep -i 8126
grep -i 'v000010ECd00008126' /lib/modules/$(uname -r)/modules.alias
```

If RTL8126 has no kernel driver:

1. Obtain temporary connectivity through Wi-Fi, USB tethering, or a supported USB Ethernet adapter.
2. Inspect the installed kernel, NVIDIA/DKMS state, Secure Boot state, and `/boot` capacity.
3. Install the supported Ubuntu 24.04 HWE kernel, not an untracked third-party driver, unless a separate decision documents why HWE is unsuitable.
4. Keep the previous kernel as a rollback option.
5. Reboot locally and verify `Kernel driver in use: r8169`.

Required verification:

```bash
uname -r
lspci -nnk -d 10ec:8126
ip -br link
sudo ethtool <ethernet-interface>
ip route
ping -c 4 <gateway-address>
ping -c 4 1.1.1.1
getent hosts archive.ubuntu.com
systemctl status ssh --no-pager
ss -lntp | grep ':22'
```

Acceptance criteria:

- ✅ RTL8126 uses the in-kernel `r8169` driver on `7.0.0-30-generic`.
- ✅ Ethernet link and DHCP persist after a controlled reboot.
- ✅ Ethernet MAC address is `9c:6b:00:b0:83:8f`.
- ✅ Router DHCP assigns the reserved Ethernet address `192.168.0.176`.
- ✅ Intended LAN address is verified as `192.168.0.176/24`.
- ✅ SSH key authentication works over Ethernet from `192.168.0.134`.
- ✅ Temporary Wi-Fi is disabled and unmanaged after Ethernet validation; the rollback file is retained as `/etc/netplan/50-cloud-init.yaml.disabled-wifi`.

## Milestone 3 — Base operating system

Completed work:

- ✅ Preserved hostname `llmsrv` and corrected the stale `/etc/hosts` alias from `llm_srv` to `llmsrv`; backup: `/etc/hosts.pre-oneagent-m3-20260823`.
- ✅ Preserved `Etc/UTC`; the system clock is synchronized, RTC is UTC, and local-time RTC mode is disabled.
- ✅ Preserved persistent locale policy `LANG=ru_RU.UTF-8`; `LC_ALL=C.UTF-8` observed during automation is a client-supplied session override, not persistent server configuration.
- ✅ Verified active and enabled `systemd-timesyncd`; synchronization uses the current network-provided NTP server with `ntp.ubuntu.com` as fallback.
- ✅ Refreshed Ubuntu package indexes; `apt-get -s upgrade` reports zero pending packages and `apt-mark showhold` is empty.
- ✅ Verified that active APT sources are the official Ubuntu archives for `noble`, `noble-updates`, `noble-backports`, and `noble-security`; no third-party source is active.
- ✅ Preserved enabled and active `unattended-upgrades` with `noble`, `noble-security`, and configured Ubuntu ESM security origins; regular `noble-updates`, proposed, and backports are not allowed for unattended installation.
- ✅ Confirmed current HWE kernel `7.0.0-30-generic`, HWE tracking packages `linux-generic-hwe-24.04` and `linux-image-generic-hwe-24.04`, and bootable rollback kernel `6.8.0-138-generic`.
- ✅ Preserved the existing `/swap.img` 8 GiB swap file; it was enabled and unused with approximately 29 GiB memory available.
- ✅ Verified capacity: root 98 GiB with 81 GiB available, `/boot` 2 GiB with 1.6 GiB available, EFI 1.1 GiB with 1.1 GiB available, and RAG 110 GiB with 109 GiB available; no LVM extension or swap resizing was required.
- ✅ Verified baseline administration tools including `curl`, `wget`, `git`, `rsync`, `jq`, `vim-tiny`, `nano`, `tmux`, `htop`, `lsof`, `pciutils`, `usbutils`, `ethtool`, `smartmontools`, and `nvme-cli`; no package installation was required.
- ✅ Verified no failed systemd units and no unexpected listeners: TCP `22` on IPv4/IPv6 for SSH, TCP/UDP `53` on loopback for the local resolver, and UDP `68` on `enp8s0` for DHCP.

The explicit `unattended-upgrade` run installed no updates, but automatically removed the unused `6.8.0-100` kernel, headers, modules, and tools. Its kernel-removal hook regenerated GRUB, and `os-prober` read the excluded `/dev/nvme1n1p2` EFI metadata to detect Windows Boot Manager. No excluded-disk modification was reported, but this read-only probe was outside the intended task scope. The required `6.8.0-138` rollback kernel remains installed and bootable. No manual `apt autoremove` was run, and no reboot was required.

Current-boot warning classification:

- Expected or non-blocking: NVIDIA out-of-tree module taint, headless audio `no codecs found`, legacy NVMe `SUBNQN`, SATA capability/quirk messages, optional unset `cron` and `smartd` environment variables, and the current `boltd` PCI-ID warning.
- Peripheral history requiring observation but not blocking this milestone: repeated USB port `3-7` enumeration failures; no failed unit or required server device loss resulted.
- Actionable and resolved: the invalid `/etc/hosts` name `llm_srv`; post-change lookup resolves `llmsrv` to `127.0.1.1`, with no new `systemd-resolved` warning.

Validation:

```bash
timedatectl
systemctl --failed
apt-mark showhold
pro security-status
journalctl -b -p warning --no-pager
```

## Milestone 4 — Storage architecture

### Required decision before formatting

The 128 GB SATA disk is intended for RAG source data. Confirm whether it is empty and may be erased. Never infer this from its size or device name.

Proposed initial layout:

- 256 GB NVMe: Ubuntu, `/opt`, model files, Open WebUI application/database data, logs, and operational tooling.
- 128 GB SATA: a dedicated filesystem mounted at `/srv/llm/rag` for RAG source documents.
- Backup: initially on the system disk only if capacity permits; durable backup requires a separate external destination.

Important constraints:

- 128 GB is not a backup and provides no redundancy.
- Open WebUI vector data is application state and must be backed up even if source documents live on the SATA disk.
- RTX 5090 makes larger models practical, but the 256 GB NVMe limits the model library; capacity guards are mandatory.

Planned work:

- ✅ Verified the RAG target as 120GB SSD serial `I38765R017695`, stable ID `/dev/disk/by-id/wwn-0x502b2a201d1c1b1a`.
- ✅ Applied the accepted clean-deployment decision; no old RAG or vector state was restored.
- ✅ Created ext4 filesystem label `llm-rag`, UUID `9a4cadeb-cb7b-4808-a83f-c259a65423d7`.
- ✅ Mounted the filesystem at `/srv/llm/rag` by UUID with `defaults,nofail,nodev,nosuid,x-systemd.device-timeout=10s`.
- ✅ Validated the reviewed `/etc/fstab` entry and `mount -a`.
- ✅ Validated `/srv/llm/rag` after reboot; approximately 109 GiB is available.
- ✅ Installed `smartmontools` 7.4 and `nvme-cli` 2.8 and restricted active `smartd` monitoring to exactly the system NVMe and RAG SATA stable IDs.
- ⬜ Create disk-usage alert thresholds for both disks.
- ⬜ Define model capacity policy and minimum free-space threshold.

Protected recovery evidence:

- ✅ The external backup filesystem UUID `98e9e57d-c234-4c76-8d15-d5f4f74a715f` remains unmounted and outside normal server-management scope.
- ✅ The Open WebUI backup manifest contains 68 entries and passed SHA-256 verification; three SQLite backups passed `PRAGMA integrity_check`.
- ✅ Old llama.cpp source/build artifacts, service configuration, CUDA packages, rootfs files, vector data, and the approximately 17.8 GiB GGUF model remain recovery evidence only and are not authorized for restoration.
- ✅ This deployment is explicitly clean: all new software, configuration, pinned versions, and validation must be produced independently.

## Milestone 5 — NVIDIA driver and CUDA

The NVIDIA driver and CUDA toolkit are separate deliverables. Do not assume that `ubuntu-drivers install` provides `nvcc`.

Planned work:

- ✅ Ubuntu recommended `nvidia-driver-595-open`; the headless deployment uses equivalent `595.84` open compute packages.
- ✅ Secure Boot implications were reviewed; the precompiled module is signed by `Canonical Ltd. Kernel Module Signing`.
- ✅ Installed `linux-modules-nvidia-595-open-generic-hwe-24.04`, `nvidia-headless-no-dkms-595-open`, and `nvidia-utils-595`.
- ✅ Rebooted and verified that RTX 5090 uses `nvidia` while `nouveau` is absent.
- ✅ Installed the minimal CUDA 13.2 build toolkit required for CUDA compilation without installing the full desktop-oriented toolkit, replacing the verified driver, or building llama.cpp.
- ✅ Installed `nvcc` 13.2.86 and GCC/G++ 13.3; `/usr/local/cuda` resolves to `/usr/local/cuda-13.2`.
- ✅ Compiled and ran a temporary CUDA Runtime API device-enumeration program with `-arch=sm_120` before and after the controlled reboot; it found one RTX 5090 with compute capability 12.0 and 33,647,230,976 bytes of global memory.
- ✅ Held `cuda-toolkit-config-common` and `cuda-toolkit-13-config-common` at 13.2.86-1 to prevent an APT upgrade to the 13.3 shared configuration while keeping versioned CUDA 13.2 selected.
- ⚠ Confirm NVIDIA modules remain operational after a future HWE kernel update.

Verified CUDA installation:

- Source: NVIDIA CUDA repository for Ubuntu 24.04 x86_64, configured by `cuda-keyring=1.1-1` with the repository keyring and NVIDIA pin priority 600.
- Selected top-level packages: `cuda-minimal-build-13-2=13.2.2-1`, the three 13.2 configuration packages at `13.2.86-1`, and the required `build-essential=12.10ubuntu1` dependency.
- Transaction result: 60 new packages, 0 upgrades, 0 removals; approximately 179 MiB downloaded and 710 MiB installed. CUDA library development packages beyond the minimal build set, including cuBLAS, remain deferred to the reproducible llama.cpp build task.
- Exact package set added by the toolkit transaction:

```text
binutils-common=2.42-4ubuntu2.10
binutils-x86-64-linux-gnu=2.42-4ubuntu2.10
binutils=2.42-4ubuntu2.10
build-essential=12.10ubuntu1
bzip2=1.0.8-5.1build0.1
cpp-13-x86-64-linux-gnu=13.3.0-6ubuntu2~24.04.1
cpp-13=13.3.0-6ubuntu2~24.04.1
cpp-x86-64-linux-gnu=4:13.2.0-7ubuntu1
cpp=4:13.2.0-7ubuntu1
cuda-cccl-13-2=13.2.86-1
cuda-compiler-13-2=13.2.2-1
cuda-crt-13-2=13.2.86-1
cuda-cudart-13-2=13.2.86-1
cuda-cudart-dev-13-2=13.2.86-1
cuda-culibos-dev-13-2=13.2.86-1
cuda-cuobjdump-13-2=13.2.86-1
cuda-cuxxfilt-13-2=13.2.86-1
cuda-driver-dev-13-2=13.2.86-1
cuda-minimal-build-13-2=13.2.2-1
cuda-nvcc-13-2=13.2.86-1
cuda-nvprune-13-2=13.2.86-1
cuda-profiler-api-13-2=13.2.86-1
cuda-tileiras-13-2=13.2.86-1
cuda-toolkit-13-2-config-common=13.2.86-1
cuda-toolkit-13-config-common=13.2.86-1
cuda-toolkit-config-common=13.2.86-1
dpkg-dev=1.22.6ubuntu6.6
g++-13-x86-64-linux-gnu=13.3.0-6ubuntu2~24.04.1
g++-13=13.3.0-6ubuntu2~24.04.1
g++-x86-64-linux-gnu=4:13.2.0-7ubuntu1
g++=4:13.2.0-7ubuntu1
gcc-13-base=13.3.0-6ubuntu2~24.04.1
gcc-13-x86-64-linux-gnu=13.3.0-6ubuntu2~24.04.1
gcc-13=13.3.0-6ubuntu2~24.04.1
gcc-x86-64-linux-gnu=4:13.2.0-7ubuntu1
gcc=4:13.2.0-7ubuntu1
libasan8=14.2.0-4ubuntu2~24.04.1
libatomic1=14.2.0-4ubuntu2~24.04.1
libbinutils=2.42-4ubuntu2.10
libcc1-0=14.2.0-4ubuntu2~24.04.1
libctf-nobfd0=2.42-4ubuntu2.10
libctf0=2.42-4ubuntu2.10
libdpkg-perl=1.22.6ubuntu6.6
libgcc-13-dev=13.3.0-6ubuntu2~24.04.1
libgomp1=14.2.0-4ubuntu2~24.04.1
libgprofng0=2.42-4ubuntu2.10
libhwasan0=14.2.0-4ubuntu2~24.04.1
libisl23=0.26-3build1.1
libitm1=14.2.0-4ubuntu2~24.04.1
liblsan0=14.2.0-4ubuntu2~24.04.1
libmpc3=1.3.1-1build1.1
libnvptxcompiler-13-2=13.2.86-1
libnvvm-13-2=13.2.86-1
libquadmath0=14.2.0-4ubuntu2~24.04.1
libsframe1=2.42-4ubuntu2.10
libstdc++-13-dev=13.3.0-6ubuntu2~24.04.1
libtsan2=14.2.0-4ubuntu2~24.04.1
libubsan1=14.2.0-4ubuntu2~24.04.1
lto-disabled-list=47
make=4.3-4.1build2
```

Validation:

```bash
nvidia-smi
lsmod | grep '^nvidia'
dkms status
nvcc --version
```

## Milestone 6 — Service accounts and directory layout

Planned work:

- ⬜ Create `/opt/src` and `/opt/open-webui` with explicit ownership.
- ⬜ Create the `/srv/llm` hierarchy.
- ⬜ Define a non-login service account for llama.cpp.
- ⬜ Grant only required read/write permissions per path.
- ⬜ Avoid recursive ownership of the entire hierarchy by an interactive user.
- ⬜ Protect configuration, database, backup, and log paths.

## Milestone 7 — llama.cpp reproducible build

Planned work:

- ⬜ Install the remaining documented build dependencies, including the CUDA 13.2 cuBLAS development package required by the selected llama.cpp CUDA backend.
- ⬜ Clone from the authoritative repository.
- ⬜ Pin and record a tested commit.
- ⬜ Configure a CUDA release build.
- ⬜ Build `llama-cli`, `llama-server`, and `llama-bench`.
- ⬜ Record compiler, CMake, CUDA, llama.cpp commit, and build options.
- ⬜ Run focused GPU inference and benchmark smoke tests.

Validation:

```bash
git -C /opt/src/llama.cpp rev-parse HEAD
/opt/src/llama.cpp/build/bin/llama-cli --version
/opt/src/llama.cpp/build/bin/llama-server --version
/opt/src/llama.cpp/build/bin/llama-bench --version
```

## Milestone 8 — Model management and RTX 5090 baseline

Planned work:

- ⬜ Define the model directory convention.
- ⬜ Record source URL, license, file size, quantization, and checksum.
- ⬜ Download one production candidate at a time.
- ⬜ Verify checksum before first load.
- ⬜ Establish RTX 5090 baselines for GPU layers, context, prompt processing, generation, VRAM, RAM, temperature, and power.
- ⬜ Select production parameters from measurements rather than the historical RTX 5070 configuration.
- ⬜ Define safe model switching and rollback.

The historical `gpu_layers = 37` value is not a production target for RTX 5090.

## Milestone 9 — llama-server production service

Planned work:

- ⬜ Store model parameters in `/etc/llama-server/`.
- ⬜ Create a complete hardened `llama-server.service`.
- ⬜ Bind only to `127.0.0.1:8080`.
- ⬜ Use the dedicated service account.
- ⬜ Add restart policy, startup ordering, limits, and logging.
- ⬜ Enable and start only after manual inference succeeds.
- ⬜ Validate health, model listing, completion, failure handling, and reboot startup.

Validation:

```bash
systemctl cat llama-server
systemctl status llama-server --no-pager
curl -fsS http://127.0.0.1:8080/health
curl -fsS http://127.0.0.1:8080/v1/models
journalctl -u llama-server -b --no-pager
```

## Milestone 10 — Docker and Open WebUI

Planned work:

- ⬜ Choose and document the Docker package source.
- ⬜ Install Docker Engine and Compose with version verification.
- ⬜ Review whether interactive users require Docker group membership.
- ⬜ Create a complete, version-controlled Compose file.
- ⬜ Pin the Open WebUI image version or digest.
- ⬜ Use host networking only after documenting the security implications.
- ⬜ Persist `/app/backend/data` under `/srv/llm/data/open-webui`.
- ⬜ Mount RAG source documents read-only.
- ⬜ Configure `http://127.0.0.1:8080/v1` as the backend.
- ⬜ Disable public signup after initial administration setup.
- ⬜ Validate container restart and host reboot.

Do not recreate an existing database or knowledge base during a recovery deployment.

## Milestone 11 — RAG foundation

Planned work:

- ✅ Validated `/srv/llm/rag` is mounted from the intended SATA ext4 filesystem by UUID.
- ⬜ Define source-document hierarchy and synchronization workflow.
- ⬜ Select and record the embedding model.
- ⬜ Record chunk size, overlap, Markdown splitting, hybrid retrieval, reranking, and PDF settings.
- ⬜ Identify and document the actual vector database engine.
- ⬜ Create or restore only the explicitly scoped knowledge base.
- ⬜ Define deterministic re-index and stale-document removal procedures.
- ⬜ Establish evaluation questions, expected evidence, and citation-quality checks.

Candidate knowledge bases are planned individually, not created in bulk:

- OneAgent Architecture;
- OneAgent Code;
- Rust Engineering;
- 1C Engineering;
- Linux Administration;
- CUDA Engineering;
- Docker Engineering;
- Java Engineering;
- Python Engineering.

## Milestone 12 — Backup and restore

Planned work:

- ⬜ Define backup scope for configuration, SQLite, vector data, RAG sources, Compose, monitoring, and documentation.
- ⬜ Use a SQLite-consistent backup method that accounts for WAL mode.
- ⬜ Restrict backup permissions.
- ⬜ Implement retention and failure reporting.
- ⬜ Verify archive readability automatically.
- ⬜ Copy critical backups to a separate physical destination.
- ⬜ Perform and document an Open WebUI restore test.
- ⬜ Perform and document a RAG restore/re-index test.

Models may be restored from verified upstream artifacts if their metadata and checksums are preserved; this does not replace backup of unique application data.

## Milestone 13 — Monitoring and log management

Planned work:

- ⬜ Implement `llm-status` and structured health checks.
- ⬜ Monitor llama.cpp, Open WebUI, Docker, GPU, filesystem mounts, disk capacity, and backup freshness.
- ⬜ Add GPU temperature, VRAM, and service-failure alerts.
- ⬜ Configure systemd timers with failure visibility.
- ⬜ Configure log rotation and retention.
- ⬜ Record baseline performance metrics.

## Milestone 14 — Nginx and network exposure

Planned work:

- ⬜ Install and validate Nginx on LAN first.
- ⬜ Proxy Open WebUI without directly exposing llama.cpp.
- ⬜ Validate request sizes, streaming responses, timeouts, and WebSocket behavior.
- ⬜ Review router forwarding and public exposure separately.
- ⬜ Verify DDNS through multiple resolvers and authoritative nameservers.
- ⬜ Verify external HTTP and the ACME challenge path.

Deferred:

- ⏸ HTTPS and Let's Encrypt until DNS is stable.

Abandoned unless explicitly reconsidered:

- ❌ Tailscale.
- ❌ WireGuard.

## Milestone 15 — Security hardening

Planned work:

- ⬜ Audit SSH configuration without risking lockout.
- ⬜ Audit UFW and all listening ports.
- ⬜ Remove obsolete VPN rules only after identifying their owners and rollback path.
- ⬜ Confirm Open WebUI signup and administrator policy.
- ⬜ Define secrets and API-key handling.
- ⬜ Protect backups and application data.
- ⬜ Review Docker privilege exposure.
- ⬜ Define security update and maintenance-window policy.

## Milestone 16 — Production acceptance

Acceptance requires a controlled reboot and all checks below:

- ⬜ Correct kernel boots and RTL8126 Ethernet works.
- ⬜ SSH works using the expected address and keys.
- ⬜ RTX 5090 and CUDA are healthy.
- ⬜ Dedicated RAG filesystem mounts by UUID.
- ⬜ llama-server reaches healthy state and exposes only loopback.
- ⬜ Open WebUI starts and reaches llama.cpp.
- ⬜ Existing or newly created scoped RAG retrieval works with useful citations.
- ⬜ Backup completes and passes integrity checks.
- ⬜ Monitoring reports healthy state and detects a controlled failure.
- ⬜ No unexpected failed units or public listening ports remain.

## Milestone 17 — Disaster recovery

Planned work:

- ⬜ Create a bare-metal rebuild checklist derived from verified deployment evidence.
- ⬜ Export all configuration and artifact versions.
- ⬜ Document recovery order and dependencies.
- ⬜ Test application and RAG recovery on clean storage.
- ⬜ Record recovery time and remaining manual steps.
- ⬜ Review the roadmap after every hardware or architecture change.

## Known technical debt and risks

- ✅ Live OS is Ubuntu Server 24.04.4 LTS and the current kernel is `7.0.0-30-generic`.
- ✅ HWE tracking packages remain installed and `6.8.0-138-generic` is retained as the bootable rollback kernel.
- ✅ Hostname `llmsrv`, `Etc/UTC`, persistent `LANG=ru_RU.UTF-8`, synchronized `systemd-timesyncd`, current APT state, and unattended security policy are verified.
- ✅ CPU is verified as AMD Ryzen 7 9850X3D.
- ✅ ASRock X870E Nova WiFi and BIOS `4.43` dated 2026-06-24 are verified.
- ✅ RTL8126 binds to `r8169` and negotiates 1 Gbps full duplex; the link partner currently limits the 5 GbE NIC to 1 Gbps.
- ✅ The new Ethernet MAC receives the intended `192.168.0.176` DHCP reservation.
- ✅ Ethernet, DHCP, route priority, DNS, Internet, and SSH persist after a controlled reboot.
- ✅ Temporary Wi-Fi is disabled, unmanaged, and has no IPv4 address or route.
- ⚠ Two 16 GB Kingston `KF580C38-16` DIMMs are configured at 8000 MT/s; this reduces CPU-offload headroom and remains a baseline stability risk.
- ⚠ The 256 GB system disk limits model-library growth.
- ✅ The system NVMe has approximately 129.83 GiB unallocated in `ubuntu-vg`; it is intentionally not extended yet.
- ⚠ The 120GB SATA RAG disk has no redundancy and is not a backup; disk-capacity alerts are still required.
- ✅ The RAG ext4 filesystem and reboot-persistent UUID mount are verified, with approximately 109 GiB available.
- ✅ Active `smartd` monitoring is restricted to exactly the two server-owned stable device IDs.
- ⚠ Before that restriction was applied, the initial default `DEVICESCAN` startup examined all attached disks; current configuration and runtime monitor only the system NVMe and RAG SATA SSD.
- ⚠ During Milestone 3, unattended removal of the unused `6.8.0-100` kernel triggered `update-grub`; its `os-prober` detected Windows Boot Manager on excluded `/dev/nvme1n1p2`. No device modification was reported, but future package-maintenance plans must account for this hook before execution.
- ✅ RTX 5090 is operational with Canonical-signed NVIDIA open driver `595.84`, 32,607 MiB VRAM, and PCIe Gen 5 x16 maximum capability.
- ✅ The minimal CUDA 13.2 build toolkit is installed from NVIDIA's Ubuntu 24.04 repository; `nvcc` 13.2.86 compiles native `sm_120` code and the post-reboot runtime smoke test enumerates the RTX 5090 successfully.
- ✅ The two shared CUDA configuration packages are held at 13.2.86-1; APT reports zero upgradeable packages and two intentionally held packages.
- ⚠ The minimal toolkit does not include cuBLAS development libraries; install the exact CUDA 13.2 package required by the selected llama.cpp build configuration during Milestone 7 rather than expanding this completed toolkit task.
- ✅ The external backup was integrity-audited, remains unmounted, and is a protected recovery artifact only; clean deployment is the accepted decision.
- ⚠ Above 4G Decoding and Resizable BAR firmware settings remain unverified.
- ⚠ DDNS stability is unverified.

## Open questions

1. Where will an off-host backup be stored?
2. What disk-capacity thresholds and model-library free-space policy should be enforced?
3. Are Above 4G Decoding and Resizable BAR enabled in firmware?
4. Will public HTTP/HTTPS access remain part of the target architecture?

## Verification command index

```bash
cat /etc/os-release
uname -a
hostnamectl
lscpu
free -h
lspci -nnk
lsblk -e7 -o NAME,PATH,SIZE,TYPE,FSTYPE,FSVER,LABEL,UUID,MOUNTPOINTS,MODEL,SERIAL
findmnt
df -hT
ip -br link
ip -br address
ip route
systemctl --failed
nvidia-smi
nvcc --version
docker version
docker compose version
systemctl status llama-server --no-pager
curl -fsS http://127.0.0.1:8080/health
curl -fsS http://127.0.0.1:8080/v1/models
```

## Last completed task

✅ Selected and installed the minimal pinned CUDA 13.2 build toolkit from NVIDIA's Ubuntu 24.04 repository, installed its required Ubuntu compiler toolchain, held the shared configuration packages against 13.3 drift, rebooted, and verified native `sm_120` compilation and CUDA Runtime API enumeration of the RTX 5090 while preserving the verified driver, kernels, Secure Boot, network, storage mounts, swap, smartd scope, and listener baseline.

## Exact next recommended task

Run **Milestone 6 — Service accounts and directory layout** as a bounded task:

1. Define the dedicated non-login llama.cpp service account and the exact ownership and permission model for `/opt/src`, `/etc/llama-server`, and the `/srv/llm` hierarchy.
2. Create only the reviewed directories and account state; do not clone llama.cpp, install remaining build libraries, download models, or start services.
3. Verify ownership, modes, traversal rights, service-account access boundaries, unchanged RAG mount identity, and preserved backup and excluded-storage scope.

## Change history

- 2026-08-23 — Rewritten as a complete post-installation deployment roadmap; added the Linux installation gate, reported ASRock X870E/RTX 5090 hardware, RTL8126 bootstrap, dedicated RAG disk planning, safety gates, validation criteria, recovery planning, and an exact next task.
- 2026-08-23 — Verified Ubuntu 24.04.4 with HWE kernel `7.0.0-30-generic`; configured RTL8126 Ethernet as `enp8s0` with DHCP route metric `100`; verified `192.168.0.176`, 1 Gbps full duplex, SSH, DNS, and Internet; retained Wi-Fi pending controlled reboot validation.
- 2026-08-23 — Completed post-reboot Ethernet validation and disabled temporary Wi-Fi; `enp8s0` is the only routed interface, while the original Wi-Fi Netplan file is retained as a rollback artifact.
- 2026-08-23 — Installed and reboot-validated the minimal headless NVIDIA `595.84` open driver stack with signed precompiled HWE modules; updated Linux firmware and AMD microcode; confirmed RTX 5090 operation and left CUDA toolkit installation as a separate task.
- 2026-08-23 — Verified motherboard, BIOS, two Kingston DIMMs configured at 8000 MT/s, AMD-V/IOMMU, GPU, NIC, server-owned storage identities, RAG mount, protected backup evidence, and restricted smartd scope; completed the Milestone 3 base OS audit, corrected `/etc/hosts`, refreshed APT state, preserved HWE tracking and rollback kernel `6.8.0-138`, recorded the unattended kernel-cleanup and `os-prober` scope deviation, and selected CUDA toolkit installation as the next bounded task.
- 2026-08-23 — Installed and pinned the minimal CUDA 13.2 build toolkit and required GCC/G++ 13.3 toolchain from the official NVIDIA and Ubuntu repositories; rejected the full toolkit expansion, deferred cuBLAS to the llama.cpp build milestone, reboot-validated native `sm_120` compilation and RTX 5090 enumeration, preserved all driver/kernel/network/storage invariants, and selected the Milestone 6 account and directory layout task next.
