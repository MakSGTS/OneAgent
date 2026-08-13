# Local Engineering Assistant — Deployment Runbook

## 1. Установка ОС

Ubuntu Server:

```
24.04 LTS
```

Обновление:

```bash
sudo apt update
sudo apt upgrade -y
```

---

# 2. NVIDIA

Установка:

```bash
sudo ubuntu-drivers install
```

Проверка:

```bash
nvidia-smi
```

---

# 3. Каталоги

Создать:

```bash
sudo mkdir -p /srv/llm/{models,data,logs,backup,monitoring,rag,prompts,benchmarks}
```

Права:

```bash
sudo chown -R $USER:$USER /srv/llm
```

---

# 4. Docker

Установка:

```bash
sudo apt install docker.io docker-compose-plugin -y
```

Добавить пользователя:

```bash
sudo usermod -aG docker $USER
```

---

# 5. llama.cpp

Зависимости:

```bash
sudo apt install git cmake build-essential -y
```

Сборка:

```bash
cd /opt/src

git clone https://github.com/ggerganov/llama.cpp

cd llama.cpp

cmake -B build -DGGML_CUDA=ON

cmake --build build --config Release -j
```

---

# 6. Модель

Разместить:

```
/srv/llm/models/qwen/
```

Пример:

```
Qwen3.6-27B-Q4_K_M.gguf
```

---

# 7. llama-server

Создать systemd:

```
/etc/systemd/system/llama-server.service
```

Основные параметры:

```
model:
Qwen GGUF

host:
127.0.0.1

port:
8080

context:
8192

GPU layers:
37

Flash Attention:
on
```

Запуск:

```bash
sudo systemctl daemon-reload

sudo systemctl enable llama-server

sudo systemctl start llama-server
```

Проверка:

```bash
curl http://127.0.0.1:8080/v1/models
```

---

# 8. Open WebUI

Docker compose:

Основные настройки:

```
network_mode: host
```

Volumes:

```
/srv/llm/data/open-webui:/app/backend/data

/srv/llm/rag:/app/backend/rag
```

API:

```
http://127.0.0.1:8080/v1
```

---

# 9. Backup

Резервировать:

```
/srv/llm/data/open-webui
```

Хранить:

```
5 дней
```

---

# 10. Monitoring

Создать:

```
/srv/llm/monitoring
```

Скрипты:

```
gpu-status.sh

llama-health.sh

system-status.sh

llm-status
```

---

# 11. Health Check

Создать:

```
llm-health.service

llm-health.timer
```

Интервал:

```
5 минут
```

---

# 12. RAG

Каталог:

```
/srv/llm/rag
```

Пример:

```
oneagent
rust
1c
linux
cuda
docker
java
```

---

# 13. Проверка системы

Команда:

```bash
llm-status
```

Должна показать:

- GPU;
- llama.cpp;
- Open WebUI;
- систему;
- Docker.

---

# Итоговая схема

```
Open WebUI
      |
      v
llama.cpp
      |
      v
Qwen GGUF


+
|
v

RAG Knowledge Bases
```щ
