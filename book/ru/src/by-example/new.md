# Создание нового проекта

Теперь, когда Вы изучили основные возможности фреймворка RTFM, Вы можете
попробовать его использовать на Вашем оборудовании следуя этим инструкциям.

1. Создайте экземпляр из шаблона [`cortex-m-quickstart`].

[`cortex-m-quickstart`]: https://github.com/rust-embedded/cortex-m-quickstart#cortex-m-quickstart

``` console
$ # например используя `cargo-generate`
$ cargo generate \
    --git https://github.com/rust-embedded/cortex-m-quickstart \
    --name app

$ # следуйте остальным инструкциям
```

2. Добавьте крейт устройства, сгенерированный с помощью [`svd2rust`] **v0.14.x**,
или библиотеку отладочной платы, у которой в зависимостях одно из устройств.
   Убедитесь, что опция `rt` крейта включена.

[`svd2rust`]: https://crates.io/crates/svd2rust

В этом примере я покажу использование крейта устройства [`lm3s6965`].
Эта библиотека не имеет Cargo-опции `rt`; эта опция всегда включена.

[`lm3s6965`]: https://crates.io/crates/lm3s6965

Этот крейт устройства предоставляет линковочный скрипт с макетом памяти
целевого устройства, поэтому `memory.x` и `build.rs` не нужно удалять.

``` console
$ cargo add lm3s6965 --vers 0.1.3

$ rm memory.x build.rs
```

3. Добавьте библиотеку `cortex-m-rtfm` как зависимость, и если необходимо,
включите опцию `timer-queue`.

``` console
$ cargo add cortex-m-rtfm --allow-prerelease --upgrade=none
```

4. Напишите программу RTFM.

Здесь я буду использовать пример `init` из библиотеки `cortex-m-rtfm`.

``` console
$ curl \
    -L https://github.com/japaric/cortex-m-rtfm/raw/v0.4.0-beta.1/examples/init.rs \
    > src/main.rs
```

Этот пример зависит от библиотеки `panic-semihosting`:

``` console
$ cargo add panic-semihosting
```

5. Соберите его, загрузите в микроконтроллер и запустите.

``` console
$ # ПРИМЕЧАНИЕ: Я раскомментировал опцию `runner` в `.cargo/config`
$ cargo run
{{#include ../../../../ci/expected/init.run}}```
