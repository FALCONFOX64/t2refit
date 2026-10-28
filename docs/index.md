# Documentation index

The hardware notes, the approved desktop list, and the lab catalog live in this repo. Read this page, then open the one you need. Five minutes is enough to see the layout.

## Start here

1. [README](../README.md) — what t2refit is, and what it will not do.
2. This index.
3. Hardware on the reference MacBookPro16,1: [hardware scan](hardware-scan.md), then [gap analysis](gap-analysis.md), then [roadmap](roadmap.md).
4. Approved desktop software for that Mac: [userspace software](userspace-software.md).
5. Fedora lab tools: [how the workspace sits beside the hardware tree](offensive/overview.md).

The hardware track records what this T2 Mac does and which existing project should own each gap. The lab track is a catalog of offensive-security tools and two read-only checkers. It does not install packages, and it does not change drivers.

## Hardware

| Page | What it is |
| --- | --- |
| [hardware-scan.md](hardware-scan.md) | What this MacBookPro16,1 did on 2026-10-06 |
| [gap-analysis.md](gap-analysis.md) | Ranked gaps and who should own them |
| [roadmap.md](roadmap.md) | Hardware phases after the scaffold |
| [userspace-software.md](userspace-software.md) | Approved desktop packages for this Mac |

Run the scanner with `./scripts/scan-t2-host.sh`. It is read-only.

## Offensive lab

| Page | What it is |
| --- | --- |
| [offensive/overview.md](offensive/overview.md) | How the lab track sits next to the T2 tree |
| [offensive/catalog.md](offensive/catalog.md) | Tool groups, one-line purpose, upstream, Fedora path, license |
| [offensive/install.md](offensive/install.md) | How to add a tool group on Fedora |
| [offensive/lab.md](offensive/lab.md) | How to run a local lab |
| [offensive/ai-tools.md](offensive/ai-tools.md) | Local models and report helpers |
| [offensive/cloud-agent.md](offensive/cloud-agent.md) | Smoke-test a Cursor cloud-agent environment |

From the repo root, list the groups:

```bash
PYTHONPATH=python/src python3 -m t2offsec groups
```

That command only reads the catalog. It does not install anything.
