use futures::{channel::mpsc::UnboundedReceiver, Stream, StreamExt, stream::select_all};

pub enum EventType<A, B> {
    A(A),
    B(B),
}

pub fn merge_receiver<'a, TA, TB>(
    ra: &'a mut UnboundedReceiver<TA>,
    rb: &'a mut UnboundedReceiver<TB>,
) -> impl Stream<Item = EventType<TA, TB>> + 'a where TA: std::marker::Send, TB: std::marker::Send {
    let a_items = ra.map(|item| EventType::A(item)).boxed();
    let b_items = rb.map(|item| EventType::B(item)).boxed();

    select_all(vec![a_items, b_items])
}
