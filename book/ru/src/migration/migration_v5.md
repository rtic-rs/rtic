# Миграция с v0.5.x на v1.0.0

Этот раздел описывает как обновиться с версии v0.5.x на v1.0.0 фреймворка RTIC.

## `Cargo.toml` - увеличьте версию

Измените версию `cortex-m-rtic` на `"0.6.0"`.

## `mod` вместо `const`

С поддержкой атрибутов над модулями трюк с `const APP` теперь не нужен.

Измените

``` rust
#[rtic::app(/* .. */)]
const APP: () = {
  [код здесь]
};
```

на

``` rust
#[rtic::app(/* .. */)]
mod app {
  [код здесь]
}
```

Так как теперь используется обычный модуль Rust, это значит, что можно использовать
обычный пользовательский код в этом модуле.
Также это значит, что `use`-выражения для ресурсов, используемые
в пользовательском коде должны быть перемещены внутрь `mod app`,
либо на них можно сослаться с помощью `super`. Например, измените:

```rust
use some_crate::some_func;

#[rtic::app(/* .. */)]
const APP: () = {
    fn func() {
        some_crate::some_func();
    }
};
```

на

```rust
#[rtic::app(/* .. */)]
mod app {
    use some_crate::some_func;

    fn func() {
        some_crate::some_func();
    }
}
```

или

```rust
use some_crate::some_func;

#[rtic::app(/* .. */)]
mod app {
    fn func() {
        super::some_crate::some_func();
    }
}
```

## Перенос диспетчеров из `extern "C"` в аргументы app.

Измените

``` rust
#[rtic::app(/* .. */)]
const APP: () = {
    [код здесь]

    // RTIC требует, чтобы неиспользуемые прерывания были задекларированы в блоке extern, когда
    // используются программные задачи; эти свободные прерывания будут использованы для управления
    // программными задачами.
    extern "C" {
        fn SSI0();
        fn QEI0();
    }
};
```

на

``` rust
#[rtic::app(/* .. */, dispatchers = [SSI0, QEI0])]
mod app {
  [код здесь]
}
```

Это работает и для ОЗУ-функций, см. examples/ramfunc.rs


## Структуры ресурсов - `#[shared]`, `#[local]`

Ранее ресурсы RTIC должны были размещаться в структуре с именем "Resources":

``` rust
struct Resources {
    // Ресурсы определяются здесь
}
```

Начиная с RTIC v1.0.0 структуры ресурсов аннотируются подобно
`#[task]`, `#[init]`, `#[idle]`: аттрибутами `#[shared]` и `#[local]`

``` rust
#[shared]
struct MySharedResources {
    // Разделяемые задачами ресурсы определены здесь
}

#[local]
struct MyLocalResources {
    // Ресурсы, определенные здесь нельзя передавать между задачами; каждый из них локальный для единственной задачи
}
```

Эти структуры разработчик может называть по своему желанию.

## `shared` и `local` аргументы в `#[task]`'ах

В v1.0.0 ресурсы разделены на `shared` ресурсы и `local` ресурсы.
`#[task]`, `#[init]` и `#[idle]` больше не имеют аргумента `resources`;
они должны использовать аргументы `shared` и `local`.

В v0.5.x:

``` rust
struct Resources {
    local_to_b: i64,
    shared_by_a_and_b: i64,
}

#[task(resources = [shared_by_a_and_b])]
fn a(_: a::Context) {}

#[task(resources = [shared_by_a_and_b, local_to_b])]
fn b(_: b::Context) {}
```

В v1.0.0:

``` rust
#[shared]
struct Shared {
    shared_by_a_and_b: i64,
}

#[local]
struct Local {
    local_to_b: i64,
}

#[task(shared = [shared_by_a_and_b])]
fn a(_: a::Context) {}

#[task(shared = [shared_by_a_and_b], local = [local_to_b])]
fn b(_: b::Context) {}
```

## Симметричные блокировки

Теперь RTIC использует симметричные блокировки, это значит, что метод `lock` нужно использовать для
всех доступов к `shared` ресурсам. Поскольку высокоприоритетные задачи имеют эксклюзивный доступ к ресурсу,
в старом коде можно было следующее:

``` rust
#[task(priority = 2, resources = [r])]
fn foo(cx: foo::Context) {
    cx.resources.r = /* ... */;
}

#[task(resources = [r])]
fn bar(cx: bar::Context) {
    cx.resources.r.lock(|r| r = /* ... */);
}
```

С симметричными блокировками нужно вызывать `lock` для обоих задач:

``` rust
#[task(priority = 2, shared = [r])]
fn foo(cx: foo::Context) {
    cx.shared.r.lock(|r| r = /* ... */);
}

#[task(shared = [r])]
fn bar(cx: bar::Context) {
    cx.shared.r.lock(|r| r = /* ... */);
}
```

Заметьте, что скорость работы не изменяется благодаря оптимизациям LLVM, которые убирают ненужные блокировки.

## Неблокирующий доступ к ресурсам

В RTIC 0.5 к ресурсам разделяемым задачами, запускаемыми с одинаковым
приоритетом, можно получить доступ *без* `lock` API.
Это все еще возможно в 0.6: ресурс `#[shared]` должен быть аннотирован
аттрибутом поля `#[lock_free]`.

v0.5 код:

``` rust
struct Resources {
    counter: u64,
}

#[task(resources = [counter])]
fn a(cx: a::Context) {
    *cx.resources.counter += 1;
}

#[task(resources = [counter])]
fn b(cx: b::Context) {
    *cx.resources.counter += 1;
}
```

v1.0 код:

``` rust
#[shared]
struct Shared {
    #[lock_free]
    counter: u64,
}

#[task(shared = [counter])]
fn a(cx: a::Context) {
    *cx.shared.counter += 1;
}

#[task(shared = [counter])]
fn b(cx: b::Context) {
    *cx.shared.counter += 1;
}
```

## нет преобразования `static mut`

`static mut` переменные больше не преобразуются в безопасные `&'static mut` ссылки.
Вместо этого синтаксиса используйте аргумент `local` в `#[init]`.

v0.5.x code:

``` rust
#[init]
fn init(_: init::Context) {
    static mut BUFFER: [u8; 1024] = [0; 1024];
    let buffer: &'static mut [u8; 1024] = BUFFER;
}
```

v1.0.0 code:

``` rust
#[init(local = [
    buffer: [u8; 1024] = [0; 1024]
//   type ^^^^^^^^^^^^   ^^^^^^^^^ initial value
])]
fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
    let buffer: &'static mut [u8; 1024] = cx.local.buffer;

    (Shared {}, Local {}, init::Monotonics())
}
```

## Init всегда возвращает поздние ресурсы

С целью сделать API более симметричным задача #[init] всегда возвращает поздние ресурсы.

С этого:

``` rust
#[rtic::app(device = lm3s6965)]
mod app {
    #[init]
    fn init(_: init::Context) {
        rtic::pend(Interrupt::UART0);
    }

    // [еще код]
}
```

на это:


``` rust
#[rtic::app(device = lm3s6965)]
mod app {
    #[shared]
    struct MySharedResources {}

    #[local]
    struct MyLocalResources {}

    #[init]
    fn init(_: init::Context) -> (MySharedResources, MyLocalResources, init::Monotonics) {
        rtic::pend(Interrupt::UART0);

        (MySharedResources, MyLocalResources, init::Monotonics())
    }

    // [more code]
}
```

## Вызов/планирование откуда угодно

С этой новой возвожностью, старый код, такой как:


``` rust
#[task(spawn = [bar])]
fn foo(cx: foo::Context) {
    cx.spawn.bar().unwrap();
}

#[task(schedule = [bar])]
fn bar(cx: bar::Context) {
    cx.schedule.foo(/* ... */).unwrap();
}
```

Теперь будет выглядеть так:

``` rust
#[task]
fn foo(_c: foo::Context) {
    bar::spawn().unwrap();
}

#[task]
fn bar(_c: bar::Context) {
    foo::schedule(/* ... */).unwrap();
}
```

Заметьте, что атрибуты `spawn` и `schedule` больше не нужны.

---

## Дополнительно

### Внешние задачи

Как программные, так и аппаратные задачи теперь можно определять вне модуля `mod app`.
Ранее это было возможно только путем реализации обертки, вызывающей реализацию задачи.

Смотреть примеры `examples/extern_binds.rs` и `examples/extern_spawn.rs`.

