# University Database API

🎓 REST API для управления базой данных университета, построенное на Rust Axum с SQLx для PostgreSQL.

## Особенности

✨ **Полное покрытие SQL возможностей:**
- Все типы JOIN (включая CROSS JOIN LATERAL)
- Операции над множествами (UNION, INTERSECT, EXCEPT)
- Предикаты и фильтры (EXISTS, IN, BETWEEN, LIKE, ILIKE)
- Функции работы с датами, строками и NULL
- Агрегатные функции с GROUP BY и HAVING

🛡️ **Безопасность и производительность:**
- Параметризованные запросы SQLx
- Пул соединений PostgreSQL
- Асинхронное выполнение

🏢 **Модульная архитектура:**
- Разделение по ролям (Администратор, Преподаватель, Студент)
- Обработка ошибок и валидация
- JSON API с стандартным форматом ответов

## Быстрый старт

### Предварительные требования

- Rust 1.70+
- PostgreSQL 12+
- Настроенная база данных университета

### Установка

1. **Клонирование репозитория:**
```bash
git clone https://github.com/danilasar/databases.git
cd databases
git checkout sqlx-backend
cd server
```

2. **Настройка окружения:**
```bash
# Создайте файл .env в папке server/
echo "DATABASE_URL=postgresql://postgres:password@localhost:5432/university" > .env
echo "FRONTEND_URL=http://localhost:3000" >> .env
echo "PORT=3001" >> .env
```

3. **Сборка и запуск:**
```bash
cargo build --release
cargo run
```

Сервер будет доступен по адресу: `http://localhost:3001`

## API эндпоинты

### Общие эндпоинты

| Метод | Путь | Описание |
|---------|------|----------|
| GET | `/api/health` | Проверка состояния сервера |
| GET | `/api/version` | Информация о версии API |
| GET | `/api/endpoints` | Полный список доступных эндпоинтов |

### Административные эндпоинты

Для аналитики и управления университетом:

| Метод | Путь | Описание | SQL функции |
|---------|------|----------|---------------|
| GET | `/api/admin/university-statistics` | Статистика по университетам | INNER/LEFT JOIN, AVG |
| GET | `/api/admin/teaching-load` | Нагрузка преподавателей | FULL JOIN, COUNT |
| GET | `/api/admin/structural-hierarchy` | Структурная иерархия | CROSS JOIN LATERAL |
| GET | `/api/admin/people-roles` | Все люди с ролями | UNION ALL |
| GET | `/api/admin/teachers-workload` | Статус нагрузки преподавателей | EXISTS, CASE |
| GET | `/api/admin/search-people/{term}` | Поиск людей | Строковые функции |
| GET | `/api/admin/age-statistics` | Статистика по возрасту | Функции даты/времени |
| GET | `/api/admin/speciality-stats` | Статистика по специальностям | GROUP BY, HAVING |

### Преподавательские эндпоинты

Для работы с студентами и предметами:

| Метод | Путь | Описание | SQL функции |
|---------|------|----------|---------------|
| GET | `/api/teacher/{id}/subjects` | Предметы преподавателя | RIGHT JOIN |
| GET | `/api/teacher/common-students/{id1}/{id2}` | Общие студенты | INTERSECT |
| GET | `/api/teacher/{id}/students-without-marks` | Студенты без оценок | EXCEPT |
| POST | `/api/teacher/{id}/students-by-criteria` | Фильтрация студентов | IN, BETWEEN |
| GET | `/api/teacher/search/{pattern}` | Поиск по паттерну | LIKE, ILIKE |
| GET | `/api/teacher/department-hierarchy` | Иерархия подразделений | Самосоединение |
| GET | `/api/teacher/{id}/info` | Информация о преподавателе | CAST, COALESCE |

### Студенческие эндпоинты

#### Расширенные аналитические запросы:

| Метод | Путь | Описание | SQL функции |
|---------|------|----------|---------------|
| GET | `/api/student/{id}/subjects-all` | Все предметы с преподавателями | LEFT JOIN |
| GET | `/api/student/{id}/classmates-with-marks` | Одногруппники с оценками | RIGHT JOIN |
| GET | `/api/student/{id}/group-performance` | Успеваемость группы | FULL JOIN |
| GET | `/api/student/{id}/subject-matrix` | Матрица студент-предмет | CROSS JOIN |
| GET | `/api/student/{id}/compare-speciality` | Сравнение со специальностью | Самосоединение |
| GET | `/api/student/{id}/activities` | Все активности студента | UNION |
| GET | `/api/student/{id}/unsettled-subjects` | Несданные предметы | EXCEPT |
| GET | `/api/student/{id}/common-subjects` | Общие предметы с группой | INTERSECT |
| GET | `/api/student/{id}/performance-check` | Проверка успеваемости | EXISTS |
| POST | `/api/student/{id}/subjects-by-criteria` | Предметы по критериям | IN, BETWEEN |
| GET | `/api/student/{id}/search/{pattern}` | Поиск по паттерну | LIKE, ILIKE |
| GET | `/api/student/{id}/age-analysis` | Анализ возраста | Функции даты |
| GET | `/api/student/{id}/semester-stats` | Статистика по семестрам | GROUP BY, HAVING |
| GET | `/api/student/{id}/formatted-info` | Форматированная информация | Строковые функции |
| GET | `/api/student/{id}/data-with-nulls` | Данные с NULL | COALESCE, NULLIF |

#### Классические запросы (обратная совместимость):

| Метод | Путь | Описание |
|---------|------|----------|
| GET | `/api/student/{id}/curriculum` | Учебный план |
| GET | `/api/student/{id}/group-mates` | Одногруппники |
| GET | `/api/student/{id}/marks` | Оценки студента |
| GET | `/api/student/{id}/current-semester/{term}` | Предметы текущего семестра |
| GET | `/api/student/{id}/speciality-info` | Информация о специальности |
| GET | `/api/student/{id}/study-progress` | Прогресс обучения |
| GET | `/api/student/{id}/teachers` | Преподаватели студента |
| GET | `/api/student/{id}/debts/{term}` | Академические долги |

## Примеры использования

### Проверка состояния
```bash
curl http://localhost:3001/api/health
# Ответ: OK
```

### Получение статистики университетов
```bash
curl http://localhost:3001/api/admin/university-statistics
```

### Поиск студентов по критериям
```bash
curl -X POST http://localhost:3001/api/teacher/1/students-by-criteria \
  -H "Content-Type: application/json" \
  -d '{
    "courses": [1, 2, 3],
    "min_age": 18,
    "max_age": 25
  }'
```

### Анализ возраста студента
```bash
curl http://localhost:3001/api/student/123/age-analysis
```

## Формат ответов

Все API ответы следуют стандартному формату:

### Успешный ответ
```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "name": "Иванов Иван",
      "course": 2
    }
  ]
}
```

### Ответ с ошибкой
```json
{
  "success": false,
  "error": "Описание ошибки",
  "code": 400
}
```

## Технические детали

### Поддерживаемые SQL возможности

#### Типы соединений
- **INNER JOIN** - Основные связи между таблицами
- **LEFT JOIN** - Включение всех записей из левой таблицы
- **RIGHT JOIN** - Включение всех записей из правой таблицы
- **FULL JOIN** - Полное внешнее соединение
- **CROSS JOIN** - Картезианское произведение
- **CROSS JOIN LATERAL** - Коррелированные подзапросы
- **Самосоединение** - Соединение таблицы с самой собой

#### Операции над множествами
- **UNION / UNION ALL** - Объединение результатов
- **INTERSECT** - Пересечение множеств
- **EXCEPT** - Разность множеств

#### Предикаты фильтрации
- **EXISTS** - Проверка существования
- **IN** - Проверка вхождения в множество
- **ANY/SOME** - Проверка любого значения
- **ALL** - Проверка всех значений
- **BETWEEN** - Проверка диапазона
- **LIKE/ILIKE** - Поиск по шаблону

#### Функции и выражения
- **CASE** - Условная логика
- **CAST, ::** - Преобразование типов
- **COALESCE, NULLIF** - Работа с NULL
- **GREATEST, LEAST** - Минимум и максимум

#### Строковые функции
- **LENGTH** - Длина строки
- **SUBSTRING** - Подстрока
- **CONCAT** - Конкатенация
- **UPPER, LOWER** - Изменение регистра
- **TRIM, LTRIM, BTRIM** - Обрезка пробелов
- **REPLACE** - Замена подстрок
- **STRPOS, POSITION** - Поиск позиции
- **OVERLAY** - Наложение строк

#### Функции даты и времени
- **NOW(), CURRENT_DATE, CURRENT_TIME** - Текущие дата/время
- **AGE()** - Вычисление возраста
- **EXTRACT(), DATE_PART()** - Извлечение частей даты
- **LOCALTIMESTAMP** - Локальное время

#### Агрегатные функции
- **COUNT, SUM, AVG** - Основные агрегаты
- **MIN, MAX** - Минимум и максимум
- **GROUP BY** - Группировка
- **HAVING** - Фильтрация групп

### Модульная структура

```
server/
├── src/
│   ├── main.rs              # Основной сервер и роутинг
│   ├── queries/
│   │   ├── mod.rs           # Модуль запросов
│   │   ├── admin.rs         # SQL запросы администратора
│   │   ├── teacher.rs       # SQL запросы преподавателя
│   │   └── student.rs       # SQL запросы студента
│   └── handlers/
│       ├── mod.rs           # Модуль обработчиков
│       ├── common.rs        # Общие обработчики
│       ├── admin.rs         # REST обработчики админа
│       ├── teacher.rs       # REST обработчики преподавателя
│       └── student.rs       # REST обработчики студента
├── Cargo.toml               # Зависимости Rust
└── README.md                # Эта документация
```

### Подключение к базе данных

Система использует переменную окружения `DATABASE_URL` для подключения к PostgreSQL.

Пул соединений настроен на максимум 20 одновременных соединений.

### CORS и безопасность

Настроен CORS для разработки с фронтендом.
Поддерживаются GET, POST, PUT, DELETE запросы.

### Мониторинг и логирование

Используется `env_logger` для логирования.
Установите `RUST_LOG=debug` для подробного логирования.

## Лицензия

MIT License - см. файл LICENSE в корне репозитория.

## Контакты

По вопросам разработки обращайтесь через Issues на GitHub.