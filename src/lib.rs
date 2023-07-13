#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod input;
mod output;
mod output_type;
// mod service;

#[doc(hidden)]
pub mod __private;
pub use async_gen;

pub use databuf;
pub use frpc_macros::*;
pub use frpc_transport_core::Transport;
pub use output::*;
// pub use service::Service;

use async_gen::GeneratorState;
use databuf::Encode;
use frpc_message::TypeId;

use std::{
    future::Future,
    io,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
};

#[doc(hidden)]
pub const DATABUF_CONFIG: u8 = databuf::config::num::LEB128 | databuf::config::len::BEU30;

macro_rules! def {
    ($(#[$doc:meta])* struct $name: ident) => {
        $(#[$doc])*
        #[derive(Debug, Default, Clone, Copy)]
        pub struct $name<T>(#[doc(hidden)] pub T);

        impl<T> Deref for $name<T> {
            type Target = T;
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<T> DerefMut for $name<T> {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

def!(
    /// Represent the state of a service. and used to share state between rpc.
    ///
    /// ```rust
    #[doc = include_str!("../example/src/stateful.rs")]
    /// ```
    struct State
);

def!(
    #[doc(hidden)]
    struct SSE
);

def!(
    /// Represent synchronous function.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use frpc::*;
    ///
    /// fn add(a: i32, b: i32) -> Return<i32> {
    ///     Return(a + b)
    /// }
    ///
    /// declare! {
    ///     service Num {
    ///         rpc add = 1;
    ///     }
    /// }
    /// ```
    struct Return
);

/// Create an async-generator, which is used to send a stream of events.
/// Also known as [Server-Sent Events (SSE)](https://en.wikipedia.org/wiki/Server-sent_events).
///
/// ```rust
#[doc = include_str!("../example/src/server_sent_events.rs")]
/// ```
#[macro_export]
macro_rules! sse {
    ($($tt:tt)*) => {
        $crate::SSE($crate::async_gen::__private::gen_inner!([$crate::async_gen] $($tt)*))
    }
}

/// Generators, also commonly referred to as coroutines.
pub trait AsyncGenerator {
    /// The type of value this generator yields.
    ///
    /// This associated type corresponds to the `yield` expression and the
    /// values which are allowed to be returned each time a generator yields.
    /// For example an iterator-as-a-generator would likely have this type as
    /// `T`, the type being iterated over.
    type Yield: Encode + frpc_message::TypeId;

    /// The type of value this generator returns.
    ///
    /// This corresponds to the type returned from a generator either with a
    /// `return` statement or implicitly as the last expression of a generator
    /// literal. For example futures would use this as `Result<T, E>` as it
    /// represents a completed future.
    type Return: Encode + frpc_message::TypeId;

    /// Resumes the execution of this generator.
    ///
    /// This function will resume execution of the generator or start execution
    /// if it hasn't already. This call will return back into the generator's
    /// last suspension point, resuming execution from the latest `yield`. The
    /// generator will continue executing until it either yields or returns, at
    /// which point this function will return.
    ///
    /// # Return value
    ///
    /// The `GeneratorState` enum returned from this function indicates what
    /// state the generator is in upon returning. If the `Yielded` variant is
    /// returned then the generator has reached a suspension point and a value
    /// has been yielded out. Generators in this state are available for
    /// resumption at a later point.
    ///
    /// If `Complete` is returned then the generator has completely finished
    /// with the value provided. It is invalid for the generator to be resumed
    /// again.
    ///
    /// # Panics
    ///
    /// This function may panic if it is called after the `Complete` variant has
    /// been returned previously. While generator literals in the language are
    /// guaranteed to panic on resuming after `Complete`, this is not guaranteed
    /// for all implementations of the `Generator` trait.
    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<GeneratorState<Self::Yield, Self::Return>>;
}

impl<G> AsyncGenerator for G
where
    G: async_gen::AsyncGenerator,
    G::Yield: Encode + TypeId,
    G::Return: Encode + TypeId,
{
    type Yield = G::Yield;
    type Return = G::Return;

    fn poll_resume(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<GeneratorState<Self::Yield, Self::Return>> {
        G::poll_resume(self, cx)
    }
}
