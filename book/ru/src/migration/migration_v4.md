# Миграция с v0.4.x на v0.5.0

Этот раздел описывает как обновить программы, написанные на RTIC v0.4.x
на версию v0.5.0 фреймворка.

## `Cargo.toml`

Во-первых, нужно обновить версию зависимости `cortex-m-rtic` до
`"0.5.0"`. Опцию `timer-queue` нужно удалить.

``` toml
[dependencies.cortex-m-rtic]
# изменить это
version = "0.4.3"

# на это
version = "0.5.0"

# и удалить Cargo feature
features = ["timer-queue"]
#           ^^^^^^^^^^^^^
```

## Аргумент `Context`

Все функции внутри элемента `#[rtic::app]` должны принимать первым аргументом
структуру `Context`. Этот тип `Context` будет содержать переменные, которые были магически
инъецированы в область видимости функции версией v0.4.x фреймворка:
`resources`, `spawn`, `schedule` -- эти переменные станут полями структуры `Context`.
Каждая функция элемента `#[rtic::app]` получит отдельный тип `Context`.

``` rust
#[rtic::app(/* .. */)]
const APP: () = {
    // change this
    #[task(resources = [x], spawn = [a], schedule = [b])]
    fn foo() {
        resources.x.lock(|x| /* .. */);
        spawn.a(message);
        schedule.b(baseline);
    }

    // into this
    #[task(resources = [x], spawn = [a], schedule = [b])]
    fn foo(mut cx: foo::Context) {
        // ^^^^^^^^^^^^^^^^^^^^

        cx.resources.x.lock(|x| /* .. */);
    //  ^^^

        cx.spawn.a(message);
    //  ^^^

        cx.schedule.b(message, baseline);
    //  ^^^
    }

    // change this
    #[init]
    fn init() {
        // ..
    }

    // into this
    #[init]
    fn init(cx: init::Context) {
        //  ^^^^^^^^^^^^^^^^^
        // ..
    }

    // ..
};
```

## Ресурсы

Синтаксис, используемый, для определения ресурсов был изменен с переменных `static mut`
на структуру `Resources`.

``` rust
#[rtic::app(/* .. */)]
const APP: () = {
    // измените это
    static mut X: u32 = 0;
    static mut Y: u32 = (); // поздний ресурс

    // на это
    struct Resources {
        #[init(0)] // <- начальное значение
        X: u32, // ПРИМЕЧАНИЕ: мы предлагаем изменить стиль именования на `snake_case`

        Y: u32, // поздний ресурс
    }

    // ..
};
```

## Периферия устройства

Если ваша программа получала доступ к периферии в `#[init]` через
переменну `device`, вам нужно будет добавить `peripherals = true` в атрибут
`#[rtic::app]`, чтобы и дальше получать доступ к периферии через поле `device` структуры `init::Context`.

Измените это:

``` rust
#[rtic::app(/* .. */)]
const APP: () = {
    #[init]
    fn init() {
        device.SOME_PERIPHERAL.write(something);
    }

    // ..
};
```

На это:

``` rust
#[rtic::app(/* .. */, peripherals = true)]
//                    ^^^^^^^^^^^^^^^^^^
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        //  ^^^^^^^^^^^^^^^^^
        cx.device.SOME_PERIPHERAL.write(something);
    //  ^^^
    }

    // ..
};
```

## `#[interrupt]` и `#[exception]`

Атрибуты `#[interrupt]` и `#[exception]` были удалены. Чтобы определять аппаратные задачи в v0.5.x
используте атрибут `#[task]` с аргументом `binds`.

Измените это:

``` rust
#[rtic::app(/* .. */)]
const APP: () = {
    // аппаратные задачи
    #[exception]
    fn SVCall() { /* .. */ }

    #[interrupt]
    fn UART0() { /* .. */ }

    // программные задачи
    #[task]
    fn foo() { /* .. */ }

    // ..
};
```

На это:

``` rust
#[rtic::app(/* .. */)]
const APP: () = {
    #[task(binds = SVCall)]
    //     ^^^^^^^^^^^^^^
    fn svcall(cx: svcall::Context) { /* .. */ }
    // ^^^^^^ мы предлагаем использовать `snake_case` имя здесь

    #[task(binds = UART0)]
    //     ^^^^^^^^^^^^^
    fn uart0(cx: uart0::Context) { /* .. */ }

    #[task]
    fn foo(cx: foo::Context) { /* .. */ }

    // ..
};
```

## `schedule`

Интерфейс `schedule` больше не требует cargo опции `timer-queue`, которая была удалена.
Чтобы использовать интерфес `schedule`, нужно сначала определить
монотонный тамер, который будет использоваьт среды выполнения, с помощью аргумента `monotonic`
атрибута `#[rtic::app]`. Чтобы продолжить использовать счетчик циклов
(CYCCNT) в качестве монотонного таймера, как было в версии v0.4.x, добавьте
аргумент `monotonic = rtic::cyccnt::CYCCNT` в атрибут `#[rtic::app]`.

Также были добавлены типы `Duration` и `Instant`, а трейт `U32Ext` был перемещен в модуль `rtic::cyccnt`.
Этот модуль доступен только на устройствах ARMv7-M+.
Удаление `timer-queue` также возвращает периферию `DWT` в структуру периферии ядра,
включить ее в работу можно внутри `init`.

Измените это:

``` rust
use rtic::{Duration, Instant, U32Ext};

#[rtic::app(/* .. */)]
const APP: () = {
    #[task(schedule = [b])]
    fn a() {
        // ..
    }
};
```

На это:

``` rust
use rtic::cyccnt::{Duration, Instant, U32Ext};
//        ^^^^^^^^

#[rtic::app(/* .. */, monotonic = rtic::cyccnt::CYCCNT)]
//                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
const APP: () = {
    #[init]
    fn init(cx: init::Context) {
        cx.core.DWT.enable_cycle_counter();
        // опционально, настройка запуска DWT без подключенного отладчика
        cx.core.DCB.enable_trace();
    }
    #[task(schedule = [b])]
    fn a(cx: a::Context) {
        // ..
    }
};
```
