
# Convert old examples to CI runner
cp ../examples/* ./src/bin/
sd "use panic_semihosting as _;" "use examples_runner as _;" src/bin/*
sd "lm3s6965" "examples_runner::pac" src/bin/*
sd "use cortex_m_semihosting::.*" "use examples_runner::{println, exit};" src/bin/*
sd "debug::exit.*" "exit();" src/bin/*
sd "hprintln" "println" src/bin/*
sd "\"\).unwrap\(\)" "\")" src/bin/*
