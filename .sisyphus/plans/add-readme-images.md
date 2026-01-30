# Add Images to README

## TL;DR

> **Quick Summary**: Улучшить контраст фото устройства и добавить их в README.md
> 
> **Deliverables**:
> - Обработанные изображения (docs/img/img1.jpg, img2.jpg)
> - Обновлённый README.md с секцией Photos
> 
> **Estimated Effort**: Quick (< 30 min)
> **Parallel Execution**: NO - sequential
> **Critical Path**: Process images → Update README → Commit

---

## Context

### Original Request
Добавить картинки устройства в README для GitHub. Изображения немного засвечены - нужно улучшить контраст.

### Current State
- Images exist at `docs/img/img1.jpg` and `docs/img/img2.jpg`
- img1.jpg: Menu screen showing ST25TB Reader options
- img2.jpg: Data view with hex dump of chip blocks
- README.md exists but has no images

---

## Work Objectives

### Core Objective
Добавить визуальную документацию в README с качественными фото устройства.

### Concrete Deliverables
- `docs/img/img1.jpg` - улучшенный контраст
- `docs/img/img2.jpg` - улучшенный контраст  
- `README.md` - новая секция "Photos" после badges

### Definition of Done
- [ ] Изображения обработаны (контраст улучшен)
- [ ] README содержит секцию Photos с двумя изображениями inline
- [ ] Изображения отображаются корректно на GitHub

### Must Have
- Улучшенная читаемость экрана на фото
- Inline layout (две картинки рядом)
- Подписи к изображениям

### Must NOT Have (Guardrails)
- Не менять размер/разрешение изображений
- Не добавлять лишние эффекты (только контраст/яркость)
- Не трогать другие секции README

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: N/A (visual task)
- **User wants tests**: NO
- **QA approach**: Manual visual verification

### Verification
- Открыть README на GitHub и убедиться что картинки отображаются
- Проверить что текст на экранах читаемый

---

## TODOs

- [ ] 1. Улучшить контраст изображений с помощью ImageMagick

  **What to do**:
  - Установить ImageMagick если не установлен: `brew install imagemagick`
  - Обработать img1.jpg: уменьшить яркость, увеличить контраст
  - Обработать img2.jpg: аналогично
  - Команда: `convert <file> -brightness-contrast -5x15 -modulate 95,115 <file>`

  **Must NOT do**:
  - Не менять размер изображений
  - Не конвертировать формат

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 2
  - **Blocked By**: None

  **References**:
  - `docs/img/img1.jpg` - Menu screen image
  - `docs/img/img2.jpg` - Data view image

  **Acceptance Criteria**:
  - [ ] img1.jpg обработан (файл изменён)
  - [ ] img2.jpg обработан (файл изменён)
  - [ ] Визуально экран читается лучше

  **Commit**: NO (группируется с Task 2)

---

- [ ] 2. Добавить секцию Photos в README.md

  **What to do**:
  - Открыть README.md
  - После строки с badges (после `![Language]...`) добавить секцию Photos
  - Использовать HTML для inline layout с подписями
  
  **Добавить этот блок после badges:**
  ```markdown
  
  ## Photos
  
  <p align="center">
    <img src="docs/img/img1.jpg" width="280" alt="Menu Screen">
    &nbsp;&nbsp;&nbsp;
    <img src="docs/img/img2.jpg" width="280" alt="Data View">
  </p>
  <p align="center">
    <em>Menu Screen</em>
    &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;
    <em>Chip Data View</em>
  </p>
  ```

  **Must NOT do**:
  - Не менять другие секции
  - Не удалять существующий контент

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 3
  - **Blocked By**: Task 1

  **References**:
  - `README.md:1-10` - Начало файла с badges

  **Acceptance Criteria**:
  - [ ] README.md содержит секцию ## Photos
  - [ ] Две картинки с путями docs/img/img1.jpg и img2.jpg
  - [ ] Подписи Menu Screen и Chip Data View

  **Commit**: YES
  - Message: `docs: add device photos to README`
  - Files: `README.md`, `docs/img/img1.jpg`, `docs/img/img2.jpg`

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 2 | `docs: add device photos to README` | README.md, docs/img/* | Visual check on GitHub |

---

## Success Criteria

### Final Checklist
- [ ] Изображения обработаны и улучшены
- [ ] README.md содержит секцию Photos
- [ ] Картинки отображаются inline (рядом)
- [ ] Коммит создан
