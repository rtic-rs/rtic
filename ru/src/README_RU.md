# Real Time For the Masses

Конкурентный фреймворк для создания систем реального времени.

## Возможности

- **Задачи** - единица конкуренции [^1]. Задачи могут *запускаться по событию*
  (в ответ на асинхронный стимул) или вызываться программно по желанию.

- **Передача сообщений** между задачами. А именно, сообщения можно передавать
  программным задачам в момент вызова.

- **Очередь таймера** [^2]. Программные задачи можно планировать на запуск в
  определенный момент в будущем. Это свойство можно использовать, чтобы
  реализовывать периодические задачи.

- Поддержка приоритетов задач, и таким образом, **вытесняющей многозадачности**.

- **Эффективное, свободное от гонок данных разделение памяти** через хорошо
  разграниченные критические секции на *основе приоритетов* [^1].

- **Выполнение без взаимной блокировки задач**, гарантированное на этапе
  компиляции. Это более сильная гарантия, чем предоставляемая
  [стандартной абстракцией `Mutex`][std-mutex].

[std-mutex]: https://doc.rust-lang.org/std/sync/struct.Mutex.html

- **Минимальные затраты на диспетчеризацию**. Диспетчер задач имеет
  минимальный след; основная часть работы по диспетчеризации делается аппаратно.

- **Высокоэффективное использование памяти**: Все задачи используют общий стек
  вызовов и нет сильной зависимости от динамического распределителя памяти.

- **Все устройства Cortex-M полностью поддерживаются**.

- Эта модель задач поддается известному анализу методом WCET (наихудшего
  времени исполнения) и техникам анализа диспетчеризации. (Хотя мы еще не
  разработали для дружественных инструментов для этого).

## Требования

- Rust 1.31.0+

- Программы нужно писать используя 2018 edition.

## [User documentation](https://japaric.github.io/cortex-m-rtfm/book)

## [API reference](https://japaric.github.io/cortex-m-rtfm/api/rtfm/index.html)

## Благодарности

Эта библиотека основана на [языке RTFM][rtfm-lang], созданном Embedded
Systems group в [Техническом Университете Luleå][ltu], под рук.
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

Все исходные тексты (включая примеры кода) лицензированы либо под:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) или
  [https://www.apache.org/licenses/LICENSE-2.0][L1])
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  [https://opensource.org/licenses/MIT][L2])

[L1]: https://www.apache.org/licenses/LICENSE-2.0
[L2]: https://opensource.org/licenses/MIT

на Ваше усмотрение.

Текст книги лицензирован по условиям лицензий
Creative Commons CC-BY-SA v4.0 ([LICENSE-CC-BY-SA](LICENSE-CC-BY-SA) или
[https://creativecommons.org/licenses/by-sa/4.0/legalcode][L3]).

[L3]: https://creativecommons.org/licenses/by-sa/4.0/legalcode

### Contribution

Если вы явно не заявляете иначе, любой взнос, преднамеренно представленный
для включения в эту работу, как определено в лицензии Apache-2.0, лицензируется, как указано выше, без каких-либо дополнительных условий.
