#![doc = include_str!("../README.md")]
// #![warn(missing_docs)]

mod input;
mod output;
mod output_type;
mod state;
// mod service;

#[doc(hidden)]
pub mod __private;
pub use async_gen;

pub use databuf;
pub use frpc_macros::*;
pub use frpc_transport_core::Transport;
pub use output::*;
pub use state::State;
// pub use service::Service;

use async_gen::GeneratorState;
use databuf::Encode;
use frpc_message::TypeId;

use std::{
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
};

#[doc(hidden)]
pub const DATABUF_CONFIG: u8 = databuf::config::num::LEB128 | databuf::config::len::BEU30;

pub struct SSE<G>(pub G);
pub struct Return<T>(pub T);

#[macro_export]
macro_rules! sse {
    ($($tt:tt)*) => {
        $crate::SSE($crate::async_gen::__private::gen_inner!([$crate::async_gen] $($tt)*))
    }
}

pub trait AsyncGenerator {
    type Yield: Encode + frpc_message::TypeId;
    type Return: Encode + frpc_message::TypeId;

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
