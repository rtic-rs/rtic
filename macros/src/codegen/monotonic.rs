use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::ast::App;

use crate::{analyze::Analysis, check::Extra, codegen::util};

/// Generates monotonic module dispatchers
pub fn codegen(app: &App, _analysis: &Analysis, _extra: &Extra) -> TokenStream2 {
    let mut monotonic_parts: Vec<_> = Vec::new();

    let tq_marker = util::timer_queue_marker_ident();

    for (_, monotonic) in &app.monotonics {
        // let instants = util::monotonic_instants_ident(name, &monotonic.ident);
        let monotonic_name = monotonic.ident.to_string();

        let tq = util::tq_ident(&monotonic_name);
        let m = &monotonic.ident;
        let m_ident = util::monotonic_ident(&monotonic_name);
        let m_isr = &monotonic.args.binds;
        let enum_ = util::interrupt_ident();
        let name_str = &m.to_string();
        let ident = util::monotonic_ident(name_str);
        let doc = &format!(
            "This module holds the static implementation for `{}::now()`",
            name_str
        );

        let (enable_interrupt, pend) = if &*m_isr.to_string() == "SysTick" {
            (
                quote!(core::mem::transmute::<_, rtic::export::SYST>(()).enable_interrupt()),
                quote!(rtic::export::SCB::set_pendst()),
            )
        } else {
            let rt_err = util::rt_err_ident();
            (
                quote!(rtic::export::NVIC::unmask(#rt_err::#enum_::#m_isr)),
                quote!(rtic::pend(#rt_err::#enum_::#m_isr)),
            )
        };

        let default_monotonic = if monotonic.args.default {
            quote!(
                #[doc(inline)]
                pub use #m::now;
                #[doc(inline)]
                pub use #m::delay;
                #[doc(inline)]
                pub use #m::timeout_at;
                #[doc(inline)]
                pub use #m::timeout_after;
            )
        } else {
            quote!()
        };

        monotonic_parts.push(quote! {
            #default_monotonic

            #[doc = #doc]
            #[allow(non_snake_case)]
            pub mod #m {
                /// Read the current time from this monotonic
                pub fn now() -> <super::super::#m as rtic::Monotonic>::Instant {
                    rtic::export::interrupt::free(|_| {
                        use rtic::Monotonic as _;
                        if let Some(m) = unsafe{ &mut *super::super::#ident.get_mut() } {
                            m.now()
                        } else {
                            <super::super::#m as rtic::Monotonic>::zero()
                        }
                    })
                }

                /// Delay
                #[inline(always)]
                #[allow(non_snake_case)]
                pub fn delay(duration: <super::super::#m as rtic::Monotonic>::Duration)
                    -> DelayFuture  {
                    let until = now() + duration;
                    DelayFuture { until, waker_storage: None }
                }

                /// Delay future.
                #[allow(non_snake_case)]
                #[allow(non_camel_case_types)]
                pub struct DelayFuture {
                    until: <super::super::#m as rtic::Monotonic>::Instant,
                    waker_storage: Option<rtic::export::IntrusiveNode<rtic::export::WakerNotReady<super::super::#m>>>,
                }
                
                impl Drop for DelayFuture {
                    fn drop(&mut self) {
                        if let Some(waker_storage) = &mut self.waker_storage {
                            rtic::export::interrupt::free(|_| unsafe {
                                let tq = &mut *super::super::#tq.get_mut();
                                tq.cancel_waker_marker(waker_storage.val.marker);
                            });
                        }
                    }
                }

                impl core::future::Future for DelayFuture {
                    type Output = ();

                    fn poll(
                        mut self: core::pin::Pin<&mut Self>,
                        cx: &mut core::task::Context<'_>
                    ) -> core::task::Poll<Self::Output> {
                        let mut s = self.as_mut();
                        let now = now();
                        let until = s.until;
                        let is_ws_none = s.waker_storage.is_none();

                        if now >= until {
                            return core::task::Poll::Ready(());
                        } else if is_ws_none {
                            rtic::export::interrupt::free(|_| unsafe {
                                let marker = super::super::#tq_marker.get().read();
                                super::super::#tq_marker.get_mut().write(marker.wrapping_add(1));

                                let nr = s.waker_storage.insert(rtic::export::IntrusiveNode::new(rtic::export::WakerNotReady {
                                    waker: cx.waker().clone(),
                                    instant: until,
                                    marker,
                                }));

                                let tq = &mut *super::super::#tq.get_mut();

                                tq.enqueue_waker(
                                    core::mem::transmute(nr), // Transmute the reference to static
                                    || #enable_interrupt,
                                    || #pend,
                                    (&mut *super::super::#m_ident.get_mut()).as_mut());
                            });
                        }

                        core::task::Poll::Pending
                    }
                }

                /// Timeout future.
                #[allow(non_snake_case)]
                #[allow(non_camel_case_types)]
                pub struct TimeoutFuture<F: core::future::Future> {
                    future: F,
                    until: <super::super::#m as rtic::Monotonic>::Instant,
                    waker_storage: Option<rtic::export::IntrusiveNode<rtic::export::WakerNotReady<super::super::#m>>>,
                }

                impl<F: core::future::Future> Drop for TimeoutFuture<F> {
                    fn drop(&mut self) {
                        if let Some(waker_storage) = &mut self.waker_storage {
                            rtic::export::interrupt::free(|_| unsafe {
                                let tq = &mut *super::super::#tq.get_mut();
                                tq.cancel_waker_marker(waker_storage.val.marker);
                            });
                        }
                    }
                }

                /// Timeout after
                #[allow(non_snake_case)]
                #[inline(always)]
                pub fn timeout_after<F: core::future::Future>(
                    future: F,
                    duration: <super::super::#m as rtic::Monotonic>::Duration
                ) -> TimeoutFuture<F> {
                    let until = now() + duration;
                    TimeoutFuture {
                        future,
                        until,
                        waker_storage: None,
                    }
                }

                /// Timeout at
                #[allow(non_snake_case)]
                #[inline(always)]
                pub fn timeout_at<F: core::future::Future>(
                    future: F,
                    instant: <super::super::#m as rtic::Monotonic>::Instant
                ) -> TimeoutFuture<F> {
                    TimeoutFuture {
                        future,
                        until: instant,
                        waker_storage: None,
                    }
                }

                impl<F> core::future::Future for TimeoutFuture<F>
                where
                    F: core::future::Future,
                {
                    type Output = Result<F::Output, super::TimeoutError>;

                    fn poll(
                        self: core::pin::Pin<&mut Self>,
                        cx: &mut core::task::Context<'_>
                    ) -> core::task::Poll<Self::Output> {
                        // SAFETY: We don't move the underlying pinned value.
                        let mut s = unsafe { self.get_unchecked_mut() };
                        let future = unsafe { core::pin::Pin::new_unchecked(&mut s.future) };
                        let now = now();
                        let until = s.until;
                        let is_ws_none = s.waker_storage.is_none();

                        match future.poll(cx) {
                            core::task::Poll::Ready(r) => {
                                if let Some(waker_storage) = &mut s.waker_storage {
                                    rtic::export::interrupt::free(|_| unsafe {
                                        let tq = &mut *super::super::#tq.get_mut();
                                        tq.cancel_waker_marker(waker_storage.val.marker);
                                    });
                                }

                                return core::task::Poll::Ready(Ok(r));
                            }
                            core::task::Poll::Pending => {
                                if now >= until {
                                    // Timeout
                                    return core::task::Poll::Ready(Err(super::TimeoutError));
                                } else if is_ws_none {
                                    rtic::export::interrupt::free(|_| unsafe {
                                        let marker = super::super::#tq_marker.get().read();
                                        super::super::#tq_marker.get_mut().write(marker.wrapping_add(1));

                                        let nr = s.waker_storage.insert(rtic::export::IntrusiveNode::new(rtic::export::WakerNotReady {
                                            waker: cx.waker().clone(),
                                            instant: until,
                                            marker,
                                        }));

                                        let tq = &mut *super::super::#tq.get_mut();

                                        tq.enqueue_waker(
                                            core::mem::transmute(nr), // Transmute the reference to static
                                            || #enable_interrupt,
                                            || #pend,
                                            (&mut *super::super::#m_ident.get_mut()).as_mut());
                                    });
                                }
                            }
                        }

                        core::task::Poll::Pending
                    }
                }
            }
        });
    }

    if monotonic_parts.is_empty() {
        quote!()
    } else {
        quote!(
            pub use rtic::Monotonic as _;

            /// Holds static methods for each monotonic.
            pub mod monotonics {
                /// A timeout error.
                #[derive(Debug)]
                pub struct TimeoutError;

                #(#monotonic_parts)*
            }
        )
    }
}
