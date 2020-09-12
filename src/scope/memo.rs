use downcast_rs::Downcast;

pub struct MemoStore<F: Fn(&Input) -> Output, Input: 'static + Eq, Output> {
    pub factory: F,
    pub cached_output: Output,
    pub cached_input: Input,
}

pub trait MemoStoreT: Downcast {

}

impl<F: Fn(&Input) -> Output + 'static, Input: 'static + Eq, Output: 'static> MemoStoreT for MemoStore<F, Input, Output> {

}

impl MemoStoreT for () {

}
impl_downcast!(MemoStoreT);