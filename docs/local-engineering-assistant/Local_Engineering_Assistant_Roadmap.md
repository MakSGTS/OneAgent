# Local Engineering Assistant — Roadmap

## Назначение проекта

Local Engineering Assistant — домашняя инженерная AI-лаборатория для разработки, анализа и эксплуатации.

Основные направления:

- Rust
- OneAgent
- 1С
- Python
- Linux
- CUDA
- Docker
- Java
- анализ технической документации

Цель:

Создать локального инженерного помощника с использованием LLM, RAG и специализированных баз знаний.

---

# Архитектура

```
Browser
   |
   v
Open WebUI
   |
   v
OpenAI Compatible API
   |
   v
llama.cpp
   |
   v
Qwen GGUF Model


+
|
v

RAG Knowledge Bases
```

---

# Текущее состояние

## Сервер

ОС:

- Ubuntu Server 24.04

Оборудование:

- AMD Ryzen 9 5950X
- RAM 64 GB
- NVIDIA RTX 5070 12 GB

В процессе:

- RTX 5090

---

# LLM Backend

## llama.cpp

Статус:

✅ Завершено

Расположение:

```
/opt/src/llama.cpp
```

Модель:

```
Qwen3.6-27B-Q4_K_M.gguf
```

Параметры:

```
alias:
qwen3.6-27b

context:
8192

GPU layers:
37

Flash Attention:
on

threads:
16

API:
localhost:8080
```

Systemd:

```
llama-server.service
```

---

# Open WebUI

Статус:

✅ Завершено

Docker:

```
open-webui
```

Подключение:

```
Open WebUI
        |
        v
llama.cpp OpenAI API
```

Профиль:

```
Local Engineering Assistant
```

---

# Профиль Local Engineering Assistant

Назначение:

Инженерный помощник.

Области:

- Rust development
- OneAgent architecture
- 1C development
- Python scripting
- Linux administration
- CUDA
- Docker
- Java
- Documentation analysis

---

# Backup

Статус:

✅ Завершено

Хранение:

```
/srv/llm/backup/open-webui
```

Период хранения:

```
5 дней
```

---

# Monitoring

Статус:

✅ Завершено

Созданы:

```
/srv/llm/monitoring

gpu-status.sh
llama-health.sh
system-status.sh
llm-status
```

---

# Health Check

Статус:

✅ Завершено

Systemd:

```
llm-health.timer
```

Интервал:

```
5 минут
```

Проверяет:

- llama.cpp
- Open WebUI
- GPU

---

# HTTPS

Статус:

⏸ Отложено

Причина:

TP-Link DNS нестабилен.

Проблема:

Let's Encrypt периодически получает:

```
NXDOMAIN
SERVFAIL
```

После стабилизации DNS:

```
certbot --nginx
```

---

# RAG

Статус:

✅ Базовая инфраструктура завершена

Каталог:

```
/srv/llm/rag
```

Структура:

```
rag

├── oneagent
├── rust
├── 1c
├── linux
├── cuda
├── docker
└── java
```

---

# OneAgent Knowledge Base

Статус:

✅ Создана

Название:

```
OneAgent Architecture
```

Источники:

```
docs/
adr/
architecture/
roadmap/
```

---

# Следующие этапы

## Этап 1

Проверка качества RAG.

Проверить:

- поиск ADR;
- понимание архитектуры;
- Semantic Model 2.0;
- Sprint 3.

---

## Этап 2

Создать:

```
OneAgent Code Knowledge Base
```

Источник:

```
oneagent/source
```

Назначение:

- анализ Rust кода;
- code review;
- подготовка задач Codex;
- архитектурный анализ.

---

## Этап 3

RTX 5090 optimization:

- увеличить GPU layers;
- увеличить context;
- проверить скорость;
- подобрать модели.

---

## Этап 4

HTTPS:

После стабилизации DNS.
