use kanal::AsyncReceiver;

// Custom PeekableReceiver to inspect events without consuming them
pub struct PeekableReceiver<T> {
    receiver: AsyncReceiver<T>,
    peeked: Option<T>,
}

impl<T> PeekableReceiver<T> {
    pub fn new(receiver: AsyncReceiver<T>) -> Self {
        Self {
            receiver,
            peeked: None,
        }
    }

    // Peek at the next event without consuming it
    pub fn peek(&mut self) -> Option<&T> {
        if self.peeked.is_none() {
            self.peeked = self.receiver.try_recv().unwrap_or(None);
        }
        self.peeked.as_ref()
    }

    // Consume the peeked event and return it
    pub fn take(&mut self) -> Option<T> {
        if let Some(item) = self.peeked.take() {
            Some(item)
        } else {
            self.receiver.try_recv().unwrap_or(None)
        }
    }
}
