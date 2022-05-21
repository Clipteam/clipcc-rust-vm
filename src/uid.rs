use std::sync::atomic::AtomicUsize;

static mut UID: AtomicUsize = AtomicUsize::new(0);

pub fn uid() -> usize {
    unsafe { UID.fetch_add(1, std::sync::atomic::Ordering::SeqCst) }
}
