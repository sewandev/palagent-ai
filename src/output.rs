use std::cell::RefCell;

thread_local! {
    pub static OUTPUT_BUFFER: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn capture_output<F: FnOnce()>(f: F) -> String {
    OUTPUT_BUFFER.with(|buf| {
        *buf.borrow_mut() = Some(String::new());
    });

    f();

    OUTPUT_BUFFER.with(|buf| buf.borrow_mut().take().unwrap_or_default())
}
