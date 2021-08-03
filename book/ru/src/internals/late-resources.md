# Поздние ресурсы

Некоторые ресурсы инициализируются во время выполнения после завершения функции `init`.
Важно то, что ресурсы (статические переменные) полностью инициализируются
до того, как задачи смогут запуститься, вот почему они должны быть инициализированы
пока прерывания отключены.

Ниже показан пример кода, генерируемого фреймворком для инициализации позних ресурсов.

``` rust
#[rtic::app(device = ..)]
mod app {
    struct Resources {
        x: Thing,
    }

    #[init]
    fn init() -> init::LateResources {
        // ..

        init::LateResources {
            x: Thing::new(..),
        }
    }

    #[task(binds = UART0, resources = [x])]
    fn foo(c: foo::Context) {
        let x: &mut Thing = c.resources.x;

        x.frob();

        // ..
    }

    // ..
}
```

Код, генерируемы фреймворком выглядит примерно так:

``` rust
fn init(c: init::Context) -> init::LateResources {
    // .. пользовательский код ..
}

fn foo(c: foo::Context) {
    // .. пользовательский код ..
}

// Public API
pub mod init {
    pub struct LateResources {
        pub x: Thing,
    }

    // ..
}

pub mod foo {
    pub struct Resources<'a> {
        pub x: &'a mut Thing,
    }

    pub struct Context<'a> {
        pub resources: Resources<'a>,
        // ..
    }
}

/// Детали реализации
mod app {
    // неинициализированная статическая переменная
    static mut x: MaybeUninit<Thing> = MaybeUninit::uninit();

    #[no_mangle]
    unsafe fn main() -> ! {
        cortex_m::interrupt::disable();

        // ..

        let late = init(..);

        // инициализация поздних ресурсов
        x.as_mut_ptr().write(late.x);

        cortex_m::interrupt::enable(); //~ compiler fence

        // исключения, прерывания и задачи могут вытеснить `main` в этой точке

        idle(..)
    }

    #[no_mangle]
    unsafe fn UART0() {
        foo(foo::Context {
            resources: foo::Resources {
                // `x` уже инициализирована к этому моменту
                x: &mut *x.as_mut_ptr(),
            },
            // ..
        })
    }
}
```

Важная деталь здесь то, что `interrupt::enable` ведет себя как *барьер компиляции*, который не дает компилятору переставить запись в `X` *после*
`interrupt::enable`. Если бы компилятор мог делать такие перестановки появились
бы гонки данных между этой записью и любой операцией `foo`, взаимодействующей с `X`.

Архитектурам с более сложным конвейером инструкций нужен барьер памяти
(`atomic::fence`) вместо compiler fence для полной очистки операции записи
перед включением прерываний. Архитектура ARM Cortex-M не нуждается в барьере памяти
в одноядерном контексте.
