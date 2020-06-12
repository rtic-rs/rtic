#![feature(prelude_import)]
//! examples/lock2.rs
#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]
#[prelude_import]
use core::prelude::v1::*;
#[macro_use]
extern crate core;
#[macro_use]
extern crate compiler_builtins;
use cortex_m_semihosting::debug;
use lm3s6965::Interrupt;
#[allow(non_snake_case)]
fn init(_: init::Context) {
    rtic::pend(Interrupt::GPIOA);
}
#[allow(non_snake_case)]
fn gpioa(mut c: gpioa::Context) {
    use rtic::Mutex as _;
    c.resources.shared.lock(|shared| {
        *shared += 1;
        rtic::pend(Interrupt::GPIOB);
    });
    debug::exit(debug::EXIT_SUCCESS);
}
#[allow(non_snake_case)]
fn gpiob(c: gpiob::Context) {
    use rtic::Mutex as _;
    *c.resources.shared += 1;
}
#[allow(non_snake_case)]
///Initialization function
pub mod init {
    /// Execution context
    pub struct Context {
        /// Core (Cortex-M) peripherals
        pub core: rtic::export::Peripherals,
    }
    impl Context {
        #[inline(always)]
        pub unsafe fn new(core: rtic::export::Peripherals) -> Self {
            Context { core }
        }
    }
}
mod resources {
    use rtic::export::Priority;
    use core::cell::Cell;
    #[allow(non_camel_case_types)]
    pub struct shared<'a> {
        priority: &'a Priority,
        locked: Cell<bool>,
    }
    impl<'a> shared<'a> {
        #[inline(always)]
        pub unsafe fn new(priority: &'a Priority) -> Self {
            shared {
                priority,
                locked: Cell::new(false),
            }
        }
        #[inline(always)]
        pub unsafe fn priority(&self) -> &Priority {
            self.priority
        }
        #[inline(always)]
        pub unsafe fn is_locked(&self) -> bool {
            self.locked.get()
        }
        pub unsafe fn lock(&self) {
            self.locked.set(true);
        }
    }
    #[allow(non_camel_case_types)]
    pub struct shared2<'a> {
        priority: &'a Priority,
        locked: Cell<bool>,
    }
    impl<'a> shared2<'a> {
        #[inline(always)]
        pub unsafe fn new(priority: &'a Priority) -> Self {
            shared2 {
                priority,
                locked: Cell::new(false),
            }
        }
        #[inline(always)]
        pub unsafe fn priority(&self) -> &Priority {
            self.priority
        }
        #[inline(always)]
        pub unsafe fn is_locked(&self) -> bool {
            self.locked.get()
        }
        pub unsafe fn lock(&self) {
            self.locked.set(true);
        }
    }
}
#[allow(non_snake_case)]
///Resources `gpioa` has access to
pub struct gpioaResources<'a> {
    pub shared: resources::shared<'a>,
    pub shared2: resources::shared2<'a>,
}
#[allow(non_snake_case)]
///Hardware task
pub mod gpioa {
    #[doc(inline)]
    pub use super::gpioaResources as Resources;
    /// Execution context
    pub struct Context<'a> {
        /// Resources this task has access to
        pub resources: Resources<'a>,
    }
    impl<'a> Context<'a> {
        #[inline(always)]
        pub unsafe fn new(priority: &'a rtic::export::Priority) -> Self {
            Context {
                resources: Resources::new(priority),
            }
        }
    }
}
#[allow(non_snake_case)]
///Resources `gpiob` has access to
pub struct gpiobResources<'a> {
    pub shared: &'a mut u32,
    pub shared2: &'a mut u32,
}
#[allow(non_snake_case)]
///Hardware task
pub mod gpiob {
    #[doc(inline)]
    pub use super::gpiobResources as Resources;
    /// Execution context
    pub struct Context<'a> {
        /// Resources this task has access to
        pub resources: Resources<'a>,
    }
    impl<'a> Context<'a> {
        #[inline(always)]
        pub unsafe fn new(priority: &'a rtic::export::Priority) -> Self {
            Context {
                resources: Resources::new(priority),
            }
        }
    }
}
/// Implementation details
const APP: () = { # [ doc = r" Always include the device crate which contains the vector table" ] use lm3s6965 as _ ; # [ allow ( non_upper_case_globals ) ] static mut shared : u32 = 0 ; impl < 'a > rtic :: Mutex for resources :: shared < 'a > { type T = u32 ; # [ inline ( always ) ] fn lock < R > ( & mut self , f : impl FnOnce ( & mut u32 ) -> R ) -> R { # [ doc = r" Priority ceiling" ] const CEILING : u8 = 2u8 ; unsafe { rtic :: export :: lock ( & mut shared , self . priority ( ) , CEILING , lm3s6965 :: NVIC_PRIO_BITS , f ) } } } # [ allow ( non_upper_case_globals ) ] static mut shared2 : u32 = 0 ; impl < 'a > rtic :: Mutex for resources :: shared2 < 'a > { type T = u32 ; # [ inline ( always ) ] fn lock < R > ( & mut self , f : impl FnOnce ( & mut u32 ) -> R ) -> R { # [ doc = r" Priority ceiling" ] const CEILING : u8 = 2u8 ; unsafe { rtic :: export :: lock ( & mut shared2 , self . priority ( ) , CEILING , lm3s6965 :: NVIC_PRIO_BITS , f ) } } } # [ allow ( non_snake_case ) ] # [ no_mangle ] unsafe fn GPIOA ( ) { const PRIORITY : u8 = 1u8 ; rtic :: export :: run ( PRIORITY , | | { crate :: gpioa ( gpioa :: Context :: new ( & rtic :: export :: Priority :: new ( PRIORITY ) ) ) } ) ; } impl < 'a > gpioaResources < 'a > { # [ inline ( always ) ] unsafe fn new ( priority : & 'a rtic :: export :: Priority ) -> Self { gpioaResources { shared : resources :: shared :: new ( priority ) , shared2 : resources :: shared2 :: new ( priority ) , } } } # [ allow ( non_snake_case ) ] # [ no_mangle ] unsafe fn GPIOB ( ) { const PRIORITY : u8 = 2u8 ; rtic :: export :: run ( PRIORITY , | | { crate :: gpiob ( gpiob :: Context :: new ( & rtic :: export :: Priority :: new ( PRIORITY ) ) ) } ) ; } impl < 'a > gpiobResources < 'a > { # [ inline ( always ) ] unsafe fn new ( priority : & 'a rtic :: export :: Priority ) -> Self { gpiobResources { shared : & mut shared , shared2 : & mut shared2 , } } } # [ no_mangle ] unsafe extern "C" fn main ( ) -> ! { let _TODO : ( ) = ( ) ; rtic :: export :: interrupt :: disable ( ) ; let mut core : rtic :: export :: Peripherals = core :: mem :: transmute ( ( ) ) ; let _ = [ ( ) ; ( ( 1 << lm3s6965 :: NVIC_PRIO_BITS ) - 1u8 as usize ) ] ; core . NVIC . set_priority ( lm3s6965 :: Interrupt :: GPIOA , rtic :: export :: logical2hw ( 1u8 , lm3s6965 :: NVIC_PRIO_BITS ) ) ; rtic :: export :: NVIC :: unmask ( lm3s6965 :: Interrupt :: GPIOA ) ; let _ = [ ( ) ; ( ( 1 << lm3s6965 :: NVIC_PRIO_BITS ) - 2u8 as usize ) ] ; core . NVIC . set_priority ( lm3s6965 :: Interrupt :: GPIOB , rtic :: export :: logical2hw ( 2u8 , lm3s6965 :: NVIC_PRIO_BITS ) ) ; rtic :: export :: NVIC :: unmask ( lm3s6965 :: Interrupt :: GPIOB ) ; core . SCB . scr . modify ( | r | r | 1 << 1 ) ; let late = crate :: init ( init :: Context :: new ( core . into ( ) ) ) ; rtic :: export :: interrupt :: enable ( ) ; loop { rtic :: export :: wfi ( ) } } };
