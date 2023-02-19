#[cfg(not(any(
    feature = "cortex_m_source_masking",
    feature = "cortex_m_basepri",
    feaute = "test_template"
)))]
compile_error!("No backend selected");

#[cfg(any(feature = "cortex_m_source_masking", feature = "cortex_m_basepri"))]
pub use cortex::*;

#[cfg(feature = "test_template")]
pub use cortex::*;

#[cfg(any(feature = "cortex_m_source_masking", feature = "cortex_m_basepri"))]
mod cortex;

#[cfg(feature = "test_template")]
mod template;
