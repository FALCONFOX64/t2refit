# AI assistants and report helpers

These are local helpers for a lab notebook. They draft and format text you already have. They are not attack agents. This repo does not document jailbreaks, unattended exploitation, or a model that runs tools against a target by itself.

The rows also live in the [catalog](catalog.md) under "AI assistants and report helpers".

## Local models

**Ollama** runs language models on the machine. Fedora 44 packages it as `ollama`. Use it to summarize notes you wrote, turn a bullet list into prose, or explain a log you already captured. It does not need a network after the model file is on disk.

**llama.cpp** (`llama-cpp` in Fedora, command `llama-cli`) runs a GGUF model in one process, without the Ollama daemon. Same job, smaller install: local drafting and log reading.

**Open WebUI** is a browser in front of a local runner such as Ollama. There is no Fedora 44 package. Upstream documents a container. Point it at the local runner only. Do not connect it to an outside model unless you mean to.

None of these three should be given a tool-calling loop that scans, exploits, or collects credentials. If a front-end offers that, leave it off.

## Report platforms

These store findings you already wrote and render a report. They do not discover hosts.

| Tool | What it is for | Fedora 44 |
| --- | --- | --- |
| SysReptor | Self-hosted pentest reports | no package; upstream container |
| Pwndoc | Self-hosted finding and report writer | no package |
| SpecterOps Ghostwriter | Self-hosted finding tracker and report generator | no package |
| Faraday | Lab notebook for findings | no package |
| Dradis Community | Self-hosted report platform | no package |

SpecterOps Ghostwriter is not the Fedora package `ghostwriter`. That package is a Markdown editor and is listed in the reporting group under its own name.

Screenshots and recordings for those reports can come from Flameshot and OBS Studio, both official Fedora packages, in the reporting group.

## What stays out

Kali does not have an "AI attack agent" metapackage, and this catalog does not add one. Projects whose purpose is to run an assessment by themselves are out of scope, including anything framed as a jailbreak or an autonomous operator.
