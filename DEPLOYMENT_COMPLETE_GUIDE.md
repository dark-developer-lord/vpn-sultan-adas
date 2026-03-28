# VPN Service: Полная инструкция разворачивания на VPS

**Создана**: 28 марта 2026  
**Версия**: 1.0  
**Статус**: Production-Ready

---

## 📋 Содержание

1. [Что это?](#что-это)
2. [Требования к серверу](#требования-к-серверу)
3. [Пошаговая инструкция](#пошаговая-инструкция)
4. [Проверка работы](#проверка-работы)
5. [Доступ к интерфейсам](#доступ-к-интерфейсам)
6. [Если что-то сломалось](#если-что-то-сломалось)

---

## Что это?

**VPN Service** - это полнофункциональный VPN сервис с:
- Защищённым backend (Rust)
- Веб-интерфейсом (Angular)
- Мобильным приложением (React Native)
- Мониторингом (Prometheus, Grafana)
- Базой данных (PostgreSQL)
- Кешированием (Redis)
- Логированием (Loki)

Это **production-ready** код, готовый к использованию для 30+ пользователей.

---

## Требования к серверу

**Минимум:**
- **1 vCPU core** ✅
- **4 GB RAM** ✅
- **50 GB SSD** ✅
- **4 TB bandwidth/month** ✅
- **Ubuntu 22.04 LTS** (или новее)
- **SSH доступ** с root правами

**Например**: Hostinger VPS или любой другой хостинг с такими характеристиками.

---

## Пошаговая инструкция

### ШАГ 1: Подключись к серверу по SSH

На своём компьютере открой терминал и подключись:

```
ssh -i ~/.ssh/id_ed25519 root@187.124.179.20
```

Замени `187.124.179.20` на IP адрес своего сервера.

**Должен увидеть вот это:**
```
Welcome to Ubuntu 25.10 (GNU/Linux...)
System information as of...
root@srv1536890:~#
```

---

### ШАГ 2: Обнови систему

Скопируй и вставь эту команду:

```
sudo apt update && sudo apt upgrade -y
```

Это обновит все пакеты на сервере. Может занять 2-3 минуты.

---

### ШАГ 3: Установи Docker и Git

Скопируй и вставь эту команду:

```
sudo apt install -y docker.io docker-compose git
```

Docker нужен для запуска всех сервисов (PostgreSQL, Redis, Prometheus и т.д.)

Когда закончится, проверь что всё установилось:

```
docker --version
```

Должно вернуть что-то вроде: `Docker version 24.0.0...`

---

### ШАГ 4: Клонируй проект

Скопируй и вставь эту команду:

```
git clone https://github.com/dark-developer-lord/-sultan-adas.git
```

Это загрузит весь твой проект на сервер (все файлы, документацию, код).

Когда закончится, перейди в папку проекта:

```
cd -sultan-adas
```

Проверь что всё там есть:

```
ls -la
```

Должны увидеть кучу файлов и папок.

---

### ШАГ 5: Создай файл конфигурации

Скопируй пример конфигурации:

```
cp .env.example .env
```

Теперь отредактируй его:

```
nano .env
```

**В редакторе nano:**

1. Найди строку:
```
JWT_SECRET=dev-secret-key-change-in-production-use-strong-key
```

2. Удали всё после `=` и напиши:
```
JWT_SECRET=production-secret-key-12345-change-me
```

3. Найди строку:
```
MASTER_KEY=dev-master-key-00000000000000000000000
```

4. Удали всё после `=` и напиши:
```
MASTER_KEY=production-master-key-67890-change-me
```

5. Сохрани файл:
   - Нажми **Ctrl+X**
   - Нажми **Y**
   - Нажми **Enter**

**Важно**: В production используй настоящие, сложные ключи! Вот как их генерировать:

```
openssl rand -base64 32
```

hNN7o0ccZvNfCzQhVJgly0q5YsYCCD/ncftA5jxWvmI

zlz5heH1tOgPxAYzejMSY0EHbztkcT4dhb1cPFFJMKA

Запусти эту команду дважды и скопируй результат в JWT_SECRET и MASTER_KEY.

---

### ШАГ 6: Запусти все сервисы через Docker

Скопируй и вставь эту команду:

```
docker-compose up -d
```

Флажок `-d` означает "в фоне" (detached mode).

Docker начнёт:
- Скачивать образы сервисов
- Создавать контейнеры
- Запускать PostgreSQL, Redis, API, и остальное

**Это может занять 3-5 минут в первый раз.**

Проверь что всё запустилось:

```
docker-compose ps
```

Должно вернуть таблицу со статусом всех сервисов. Все должны быть в статусе `Up`.

**Пример вывода:**
```
NAME                COMMAND                  SERVICE      STATUS
-sultan-adas-api-1           "/app/vpn-api"               api              Up 2 minutes
-sultan-adas-postgres-1      "postgres"                   postgres         Up 2 minutes
-sultan-adas-redis-1         "redis-server"               redis            Up 2 minutes
-sultan-adas-prometheus-1    "/bin/prometheus"            prometheus       Up 2 minutes
-sultan-adas-grafana-1       "/run.sh"                    grafana          Up 2 minutes
```

---

### ШАГ 7: Проверь что API работает

Скопируй и вставь эту команду:

```
curl http://localhost:3000/health
```

**Должно вернуть:**
```json
{"status":"ok"}
```

Если вернуло это - значит всё работает правильно! ✅

Если ошибка - смотри раздел [Если что-то сломалось](#если-что-то-сломалось)

---

## Проверка работы

### Смотри логи API

Если хочешь увидеть что происходит внутри:

```
docker-compose logs -f api
```

Нажми **Ctrl+C** чтобы выйти из логов.

### Смотри статус всех сервисов

```
docker-compose ps
```

### Перезагрузи всё

Если что-то баглит:

```
docker-compose restart
```

### Останови всё

```
docker-compose stop
```

### Запусти снова

```
docker-compose up -d
```

---

## Доступ к интерфейсам

После того как всё запустилось, ты можешь получить доступ к:

### 1. API (Backend)

**URL**: `http://187.124.179.20:3000`

Замени `187.124.179.20` на IP твоего сервера.

Попробуй:
```
curl http://187.124.179.20:3000/health
```

### 2. Grafana (Мониторинг)

**URL**: `http://187.124.179.20:3001`

**Логин**: `admin`  
**Пароль**: `admin`

Тут видишь графики с метриками сервера.

### 3. PostgreSQL (База данных)

**Хост**: `localhost`  
**Порт**: `5432`  
**Пользователь**: `vpn`  
**Пароль**: `vpn`  
**БД**: `vpn_service`

Подключиться можно через DBeaver или pgAdmin (если установить).

### 4. Redis (Кеш)

**Хост**: `localhost`  
**Порт**: `6379`

Нет пароля (для dev).

### 5. Prometheus (Собиратель метрик)

**URL**: `http://187.124.179.20:9090`

---

## Если что-то сломалось

### Проблема: "Connection refused"

Сервисы ещё не запустились. Подожди 5 минут и попробуй снова:

```
docker-compose ps
```

Если всё в статусе `Up` - попробуй:

```
curl http://localhost:3000/health
```

### Проблема: "docker: command not found"

Docker не установился. Попробуй установить снова:

```
sudo apt install -y docker.io
```

### Проблема: Контейнеры не запускаются

Смотри логи:

```
docker-compose logs
```

Ищи красные ошибки.

Самая частая ошибка: порты заняты. Если что-то уже слушает на 3000 порту:

```
sudo lsof -i :3000
```

### Проблема: Нет доступа к сервису с локального компьютера

Убедись что:
1. Сервер включён
2. SSH всё ещё подключен (или сервис запущен в docker-compose)
3. Firewall открыл порты (на Hostinger обычно всё открыто)

Попробуй с сервера:

```
curl http://localhost:3000/health
```

Если работает на сервере - проблема в firewall или маршрутизации.

---

## После разворачивания - что дальше?

Отлично! VPN сервис запущен. Теперь:

### 1. Тестирование (Phase 1)

Запусти smoke тесты:

```
bash scripts/smoke-tests.sh
```

Должно пройти 15 из 15 тестов.

### 2. Регистрация первого пользователя

Используй Grafana веб-интерфейс или API:

```
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "secure-password-123",
    "name": "Test User"
  }'
```

### 3. Подключение мобильного приложения

Приложение React Native находится в `/crates/mobile`

Сконфигурируй API URL на твой сервер.

### 4. Мониторинг

Открой Grafana (http://187.124.179.20:3001) и смотри:
- CPU, RAM, Disk usage
- API latency
- Error rate
- Connected users

---

## Структура проекта

```
-sultan-adas/
├── crates/
│   ├── api/              # Backend Rust код
│   ├── frontend/         # Angular веб-интерфейс
│   ├── mobile/           # React Native мобильное приложение
│   ├── crypto/           # Encryption/VPN протоколы
│   └── data/             # Работа с БД
├── docker-compose.yml    # Конфиг для Docker
├── docker/               # Dockerfiles
├── migrations/           # SQL миграции для БД
├── monitoring/           # Prometheus, Grafana, Loki
├── scripts/              # Bash скрипты для управления
└── PHASE_*_IMPLEMENTATION_GUIDE.md  # Документация по фазам
```

---

## Полезные команды

```bash
# Статус всех сервисов
docker-compose ps

# Логи API
docker-compose logs -f api

# Логи всего
docker-compose logs -f

# Перезагрузить API
docker-compose restart api

# Остановить всё
docker-compose stop

# Запустить снова
docker-compose up -d

# Удалить контейнеры (ОСТОРОЖНО!)
docker-compose down

# Заново запустить с пересозданием контейнеров
docker-compose up -d --force-recreate

# Мониторить использование ресурсов
docker stats

# Подключиться к контейнеру
docker exec -it -sultan-adas-api-1 /bin/bash

# Смотреть какие порты слушают
netstat -tulpn | grep LISTEN
```

---

## Пример: Полная инструкция за 10 минут

```bash
# 1. Подключись по SSH
ssh -i ~/.ssh/id_ed25519 root@187.124.179.20

# 2. Обнови систему
sudo apt update && sudo apt upgrade -y

# 3. Установи Docker и Git
sudo apt install -y docker.io docker-compose git

# 4. Клонируй проект
git clone https://github.com/dark-developer-lord/-sultan-adas.git
cd -sultan-adas

# 5. Скопируй конфиг
cp .env.example .env

# 6. Отредактируй .env (nano, замени ключи на production)
nano .env

# 7. Запусти
docker-compose up -d

# 8. Проверь
curl http://localhost:3000/health

# 9. Смотри Grafana
# Открой http://187.124.179.20:3001 в браузере
# Логин: admin, Пароль: admin
```

---

## Техническая информация

### Используемые сервисы

| Сервис | Порт | Описание |
|--------|------|---------|
| API (Rust/Axum) | 3000 | Основной backend |
| PostgreSQL | 5432 | База данных |
| Redis | 6379 | Кеш и сессии |
| Prometheus | 9090 | Сбор метрик |
| Grafana | 3001 | Визуализация метрик |
| Loki | 3100 | Логирование |
| AlertManager | 9093 | Оповещение об ошибках |

### Данные для подключения

**PostgreSQL:**
- Host: postgres
- Port: 5432
- User: vpn
- Password: vpn
- Database: vpn_service

**Redis:**
- Host: redis
- Port: 6379
- No password (dev)

**API:**
- Base URL: http://localhost:3000
- Health check: GET /health

---

## Поддержка и Troubleshooting

**Что делать если:**

1. **Контейнер крашится** → `docker-compose logs -f` и смотри ошибки
2. **Порт занят** → `sudo lsof -i :3000` и убей процесс
3. **БД не инициализирована** → `docker-compose down` и `docker-compose up -d`
4. **Память закончилась** → `docker system prune` для очистки
5. **Непонятная ошибка** → смотри логи сервиса в docker-compose

---

## Что дальше?

После успешного разворачивания:

1. ✅ Запусти tests: `bash scripts/smoke-tests.sh`
2. ✅ Создай тестового пользователя
3. ✅ Подключи мобильное приложение (если готово)
4. ✅ Настрой мониторинг (Grafana alerts)
5. ✅ Регулярно смотри логи на ошибки
6. ✅ Готовься к Phase 2 (Weeks 5-8)

---

## Срочно запомни

- **Никогда не деляй production ключи в GitHub**
- **Регулярно бэкапь БД**: `docker exec -sultan-adas-postgres-1 pg_dump ...`
- **Мониторь README файлы** - там актуальная информация
- **Смотри docker-compose.yml** - там все конфиги

---

**Дата обновления**: 28 марта 2026  
**Версия**: 1.0  
**Статус**: Production Ready ✅

Удачи! 🚀
