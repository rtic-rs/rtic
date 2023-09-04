fn main() {
    // feature=["stm32g081kb"] etc.
    let stm32_chip: Vec<_> = std::env::vars()
        .map(|(a, _)| a)
        .filter(|x| {
            !x.starts_with("CARGO_FEATURE_STM32_METAPAC")
                && !x.starts_with("CARGO_FEATURE_STM32_TIM")
                && x.starts_with("CARGO_FEATURE_STM32")
        })
        .collect();

    match stm32_chip.len() {
        0 => {
            // Not using stm32.
        }
        1 => {
            // Allows to just use #[cfg(stm32)] if one of the stm32 chips is used.
            println!("cargo:rustc-cfg=stm32");
        }
        _ => panic!("multiple stm32xx definitions {:?}", stm32_chip),
    }
}
