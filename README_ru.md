# Real-Time Interrupt-driven Concurrency

Конкурентный фреймворк для создания систем реального времени.

Также известный как Real-Time For the Masses.

[![crates.io](https://img.shields.io/crates/v/cortex-m-rtic)](https://crates.io/crates/cortex-m-rtic)
[![docs.rs](https://docs.rs/cortex-m-rtic/badge.svg)](https://docs.rs/cortex-m-rtic)
[![book](https://img.shields.io/badge/web-rtic.rs-red.svg?style=flat&label=book&colorB=d33847)](https://rtic.rs/)
[![rustc](https://img.shields.io/badge/rustc-1.36+-lightgray.svg)](https://github.com/rust-lang/rust/releases/tag/1.36.0)
[![matrix](https://img.shields.io/matrix/rtic:matrix.org)](https://matrix.to/#/#rtic:matrix.org)
[![Meeting notes](https://hackmd.io/badge.svg)](https://hackmd.io/@xmis9JvZT8Gvo9lOEKyZ4Q/SkBJKsjuH)

## Возможности

- **Задачи** как единица конкуренции [^1]. Задачи могут *запускаться от событий*
  (срабатывать в ответ на асинхронные воздействия) или вызываться по запросу программы.

- **Передача сообщений** между задачами. Если точнее, сообщения можно передавать
  программным задачам в момент вызова.

- **Очередь таймера** [^2]. Программные задачи можно планировать на запуск в определенный
  момент в будущем. Эту возможность можно использовать для создания периодических задач.

- Поддержка приоритета задач, и, как результат, **вытесняющей многозадачности**.

- **Эффективное, избавленное от гонок данных, разделение ресурсов** благодаря легкому
  разбиению на *основанные на приоритетах* критические секции [^1].

- **Выполнение без Deadlock**, гарантируемое на этапе компиляции. Данная гарантия строже,
  чем та, что предоставляется [стандартный абтракцией `Mutex`][std-mutex].

[std-mutex]: https://doc.rust-lang.org/std/sync/struct.Mutex.html

- **Минимальные расходы на диспетчеризацию**. Диспетчер задач иммет минимальную программную
  базу; основная работа по диспетчеризации происходит аппаратно.

- **Высокоэффективное использование памяти**: Все задачи разделяют единый стек вызовов и
  отсутствует ресурсоемкая зависисмость от динамического аллокатора.

- **Все Cortex-M устройства полностью поддерживаются**.

- К такой модели задач можно применять так называемый анализ WCET (Наихудшего времени выполнения),
  а также техники анализа диспетчеризации. (Хотя мы еще не разработали дружественный к Rust'у
  инструментарий для этого.)

## Требования

- Rust 1.51.0+

- Приложения должны быть написаны в редакции 2018.

## [Руководство пользователя](https://rtic.rs) - [(Версия в разработке)](https://rtic.rs/dev)

## [Документация пользователя](https://rtic.rs)

## [Справочник по API](https://rtic.rs/stable/api/)

## [Сборник примеров, предоставляемы сообществом][examples]

[examples]: https://github.com/rtic-rs/rtic-examples

## Чат

Присоединяйтесь к нам, чтобы говорить о RTIC [в Matrix-комнате][matrix-room].

Записи еженедельных собраний можно найти в [HackMD][hackmd]

[matrix-room]: https://matrix.to/#/#rtic:matrix.org
[hackmd]: https://hackmd.io/@xmis9JvZT8Gvo9lOEKyZ4Q/SkBJKsjuH

## Внести вклад

Новые возможности и большие изменения следует проводить через процесс RFC в
[соответствующем RFC-репозитории][rfcs].

[rfcs]: https://github.com/rtic-rs/rfcs

## Благодарности

Этот крейт основан на [языке Real-Time For the Masses][rtfm-lang], созданном Embedded
Systems group в [Техническом Университете Luleå][ltu], под руководством
[Prof. Per Lindgren][per].

[rtfm-lang]: http://www.rtfm-lang.org/
[ltu]: https://www.ltu.se/?l=en
[per]: https://www.ltu.se/staff/p/pln-1.11258?l=en

## Ссылки

[^1]: Eriksson, J., Häggström, F., Aittamaa, S., Kruglyak, A., & Lindgren, P.
   (2013, June). Real-time for the masses, step 1: Programming API and static
   priority SRP kernel primitives. In Industrial Embedded Systems (SIES), 2013
   8th IEEE International Symposium on (pp. 110-113). IEEE.

[^2]: Lindgren, P., Fresk, E., Lindner, M., Lindner, A., Pereira, D., & Pinho,
   L. M. (2016). Abstract timers and their implementation onto the arm cortex-m
   family of mcus. ACM SIGBED Review, 13(1), 48-53.

## Лицензия

Все исходные тексты (включая примеры кода) лицензированы под одной из лицензий:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) или
  [https://www.apache.org/licenses/LICENSE-2.0][L1])
- MIT license ([LICENSE-MIT](LICENSE-MIT) или
  [https://opensource.org/licenses/MIT][L2])

[L1]: https://www.apache.org/licenses/LICENSE-2.0
[L2]: https://opensource.org/licenses/MIT

на ваш выбор.

Текст книги лицензирован по условиям лицензий
Creative Commons CC-BY-SA v4.0 ([LICENSE-CC-BY-SA](LICENSE-CC-BY-SA) или
[https://creativecommons.org/licenses/by-sa/4.0/legalcode][L3]).

[L3]: https://creativecommons.org/licenses/by-sa/4.0/legalcode

### Условия участия

Если вы не укажете этого отдельно, любой вклад, который вы предоставите в эту работу,
как указано в тексте лицензии Apache-2.0, будет лицензирован по условиям,
указанным выше, без каких-либо дополнительных условий.
