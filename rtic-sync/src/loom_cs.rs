//! A loom-based implementation of CriticalSection, effectively copied from the critical_section::std module.

use core::cell::RefCell;
use core::mem::MaybeUninit;

use loom::cell::Cell;
use loom::sync::{Mutex, MutexGuard};

loom::lazy_static! {
    static ref GLOBAL_MUTEX: Mutex<()> = Mutex::new(());
    // This is initialized if a thread has acquired the CS, uninitialized otherwise.
    static ref GLOBAL_GUARD: RefCell<MaybeUninit<MutexGuard<'static, ()>>> = RefCell::new(MaybeUninit::uninit());
}

loom::thread_local!(static IS_LOCKED: Cell<bool> = Cell::new(false));

struct StdCriticalSection;
critical_section::set_impl!(StdCriticalSection);

unsafe impl critical_section::Impl for StdCriticalSection {
    unsafe fn acquire() -> bool {
        // Allow reentrancy by checking thread local state
        IS_LOCKED.with(|l| {
            if l.get() {
                // CS already acquired in the current thread.
                return true;
            }

            // Note: it is fine to set this flag *before* acquiring the mutex because it's thread local.
            // No other thread can see its value, there's no potential for races.
            // This way, we hold the mutex for slightly less time.
            l.set(true);

            // Not acquired in the current thread, acquire it.
            let guard = match GLOBAL_MUTEX.lock() {
                Ok(guard) => guard,
                Err(err) => {
                    // Ignore poison on the global mutex in case a panic occurred
                    // while the mutex was held.
                    err.into_inner()
                }
            };
            GLOBAL_GUARD.borrow_mut().write(guard);

            false
        })
    }

    unsafe fn release(nested_cs: bool) {
        if !nested_cs {
            // SAFETY: As per the acquire/release safety contract, release can only be called
            // if the critical section is acquired in the current thread,
            // in which case we know the GLOBAL_GUARD is initialized.
            //
            // We have to `assume_init_read` then drop instead of `assume_init_drop` because:
            // - drop requires exclusive access (&mut) to the contents
            // - mutex guard drop first unlocks the mutex, then returns. In between those, there's a brief
            //   moment where the mutex is unlocked but a `&mut` to the contents exists.
            // - During this moment, another thread can go and use GLOBAL_GUARD, causing `&mut` aliasing.
            #[allow(let_underscore_lock)]
            let _ = GLOBAL_GUARD.borrow_mut().assume_init_read();

            // Note: it is fine to clear this flag *after* releasing the mutex because it's thread local.
            // No other thread can see its value, there's no potential for races.
            // This way, we hold the mutex for slightly less time.
            IS_LOCKED.with(|l| l.set(false));
        }
    }
}
