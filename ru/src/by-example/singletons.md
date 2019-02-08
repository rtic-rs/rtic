# Одиночки

Атрибут `app` знает о библиотеке [`owned-singleton`] и её атрибуте [`Singleton`].
Когда этот атрибут применяется к одному из ресурсов, рантайм производит для Вас
`unsafe` инициализацию одиночки, проверяя, что только один экземпляр одиночки
когда-либо создан.

[`owned-singleton`]: ../../api/owned_singleton/index.html
[`Singleton`]: ../../api/owned_singleton_macros/attr.Singleton.html

Заметьте, что когда Вы используете атрибут `Singleton`, Вым нужно иметь
`owned_singleton` в зависимостях.

Ниже, в примере, использован атрибут `Singleton` на куске памяти, а затем
использован экземпляр одиночки как фиксированный по размеру пул памяти,
используя одну из абстракций [`alloc-singleton`].

[`alloc-singleton`]: https://crates.io/crates/alloc-singleton

``` rust
{{#include ../../../examples/singleton.rs}}
```

``` console
$ cargo run --example singleton
{{#include ../../../ci/expected/singleton.run}}```
