use std::future::Future;

pub trait AsyncFnOnce<Args> {
    type Output;
    type Future: Future<Output = Self::Output> + Send;
    fn call_once(self, _: Args) -> Self::Future;
}

pub trait AsyncFnMut<Args>: AsyncFnOnce<Args> {
    fn call_mut(&mut self, args: Args) -> Self::Future;
}

pub trait AsyncFn<Args>: AsyncFnMut<Args> {
    fn call(&self, args: Args) -> Self::Future;
}

impl<Func, Args> AsyncFnOnce<Args> for Func
where
    Func: crate::fn_trait::FnOnce<Args>,
    Func::Output: Future + Send,
{
    type Output = <Func::Output as Future>::Output;
    type Future = Func::Output;

    fn call_once(self, args: Args) -> Self::Future {
        Func::call_once(self, args)
    }
}

impl<Func, Args> AsyncFnMut<Args> for Func
where
    Func: crate::fn_trait::FnMut<Args>,
    Func::Output: Future + Send,
{
    fn call_mut(&mut self, args: Args) -> Self::Future {
        Func::call_mut(self, args)
    }
}

impl<Func, Args> AsyncFn<Args> for Func
where
    Func: crate::fn_trait::Fn<Args>,
    Func::Output: Future + Send,
{
    fn call(&self, args: Args) -> Self::Future {
        Func::call(self, args)
    }
}
