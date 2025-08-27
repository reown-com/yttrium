use {
    std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    },
    tokio::sync::mpsc,
};

pub struct PriorityReceiver<T> {
    priority_rx: mpsc::UnboundedReceiver<T>,
    normal_rx: mpsc::UnboundedReceiver<T>,
}

impl<T> PriorityReceiver<T> {
    pub fn new(
        priority_rx: mpsc::UnboundedReceiver<T>,
        normal_rx: mpsc::UnboundedReceiver<T>,
    ) -> Self {
        Self { priority_rx, normal_rx }
    }

    pub async fn recv(&mut self) -> Option<T> {
        PriorityRecvFuture { receiver: self }.await
    }

    pub fn close(&mut self) {
        self.priority_rx.close();
        self.normal_rx.close();
    }
}

struct PriorityRecvFuture<'a, T> {
    receiver: &'a mut PriorityReceiver<T>,
}

impl<'a, T> Future for PriorityRecvFuture<'a, T> {
    type Output = Option<T>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let mut priority_closed = false;
        
        // Always try priority first
        match self.receiver.priority_rx.poll_recv(cx) {
            Poll::Ready(Some(msg)) => return Poll::Ready(Some(msg)),
            Poll::Ready(None) => {
                // Priority channel closed, but continue to check normal channel
                priority_closed = true;
            }
            Poll::Pending => {} // Continue to normal channel
        }

        // Then try normal channel
        match self.receiver.normal_rx.poll_recv(cx) {
            Poll::Ready(Some(msg)) => Poll::Ready(Some(msg)),
            Poll::Ready(None) => {
                // Normal channel closed - return None only if both are closed
                // If priority is still open, we should keep waiting
                if priority_closed {
                    Poll::Ready(None) // Both channels closed
                } else {
                    Poll::Pending // Priority still open, keep waiting
                }
            }
            Poll::Pending => Poll::Pending, // At least one channel is still open and pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_priority_messages_received_first() {
        let (priority_tx, priority_rx) = mpsc::unbounded_channel();
        let (normal_tx, normal_rx) = mpsc::unbounded_channel();
        let mut receiver = PriorityReceiver::new(priority_rx, normal_rx);

        // Send normal message first
        normal_tx.send("normal").unwrap();
        // Send priority message after
        priority_tx.send("priority").unwrap();

        // Priority message should be received first
        let result = receiver.recv().await;
        assert_eq!(result, Some("priority"));

        // Then normal message
        let result = receiver.recv().await;
        assert_eq!(result, Some("normal"));
    }

    #[tokio::test]
    async fn test_normal_messages_when_no_priority() {
        let (_priority_tx, priority_rx) = mpsc::unbounded_channel();
        let (normal_tx, normal_rx) = mpsc::unbounded_channel();
        let mut receiver = PriorityReceiver::new(priority_rx, normal_rx);

        // Send only normal messages
        normal_tx.send("normal1").unwrap();
        normal_tx.send("normal2").unwrap();

        // Should receive normal messages in order
        let result = receiver.recv().await;
        assert_eq!(result, Some("normal1"));

        let result = receiver.recv().await;
        assert_eq!(result, Some("normal2"));
    }

    #[tokio::test]
    async fn test_mixed_priority_and_normal_messages() {
        let (priority_tx, priority_rx) = mpsc::unbounded_channel();
        let (normal_tx, normal_rx) = mpsc::unbounded_channel();
        let mut receiver = PriorityReceiver::new(priority_rx, normal_rx);

        // Send mixed messages
        normal_tx.send("normal1").unwrap();
        priority_tx.send("priority1").unwrap();
        normal_tx.send("normal2").unwrap();
        priority_tx.send("priority2").unwrap();

        // Should receive all priority messages first
        let result = receiver.recv().await;
        assert_eq!(result, Some("priority1"));

        let result = receiver.recv().await;
        assert_eq!(result, Some("priority2"));

        // Then normal messages
        let result = receiver.recv().await;
        assert_eq!(result, Some("normal1"));

        let result = receiver.recv().await;
        assert_eq!(result, Some("normal2"));
    }

    #[tokio::test]
    async fn test_receiver_returns_none_when_channels_closed() {
        let (priority_tx, priority_rx) = mpsc::unbounded_channel::<String>();
        let (normal_tx, normal_rx) = mpsc::unbounded_channel();
        let mut receiver = PriorityReceiver::new(priority_rx, normal_rx);

        // Drop senders to close channels
        drop(priority_tx);
        drop(normal_tx);

        // Should return None when both channels are closed
        let result = receiver.recv().await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_priority_channel_closed_normal_still_works() {
        let (priority_tx, priority_rx) = mpsc::unbounded_channel();
        let (normal_tx, normal_rx) = mpsc::unbounded_channel();
        let mut receiver = PriorityReceiver::new(priority_rx, normal_rx);

        // Close priority channel
        drop(priority_tx);
        
        // Send normal message
        normal_tx.send("normal").unwrap();

        // Should still receive normal messages
        let result = receiver.recv().await;
        assert_eq!(result, Some("normal"));
    }

    #[tokio::test]
    async fn test_normal_channel_closed_priority_still_works() {
        let (priority_tx, priority_rx) = mpsc::unbounded_channel();
        let (normal_tx, normal_rx) = mpsc::unbounded_channel();
        let mut receiver = PriorityReceiver::new(priority_rx, normal_rx);

        // Close normal channel
        drop(normal_tx);
        
        // Send priority message
        priority_tx.send("priority").unwrap();

        // Should still receive priority messages
        let result = receiver.recv().await;
        assert_eq!(result, Some("priority"));
    }

    #[tokio::test]
    async fn test_close_method() {
        let (priority_tx, priority_rx) = mpsc::unbounded_channel();
        let (normal_tx, normal_rx) = mpsc::unbounded_channel();
        let mut receiver = PriorityReceiver::new(priority_rx, normal_rx);

        // Send some messages before closing
        priority_tx.send("priority").unwrap();
        normal_tx.send("normal").unwrap();

        // Close the receiver
        receiver.close();

        // Should still receive messages that were already queued
        let result = receiver.recv().await;
        assert_eq!(result, Some("priority"));

        let result = receiver.recv().await;
        assert_eq!(result, Some("normal"));

        // No more messages should be received
        let result = receiver.recv().await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_recv_pending_when_no_messages() {
        let (_priority_tx, priority_rx) = mpsc::unbounded_channel::<i32>();
        let (_normal_tx, normal_rx) = mpsc::unbounded_channel::<i32>();
        let mut receiver = PriorityReceiver::new(priority_rx, normal_rx);

        // Should timeout when no messages are available
        let result = timeout(Duration::from_millis(100), receiver.recv()).await;
        assert!(result.is_err()); // Should timeout
    }

    #[tokio::test]
    async fn test_multiple_priority_messages_preserve_order() {
        let (priority_tx, priority_rx) = mpsc::unbounded_channel();
        let (_normal_tx, normal_rx) = mpsc::unbounded_channel();
        let mut receiver = PriorityReceiver::new(priority_rx, normal_rx);

        // Send multiple priority messages
        for i in 1..=5 {
            priority_tx.send(i).unwrap();
        }

        // Should receive them in order
        for i in 1..=5 {
            let result = receiver.recv().await;
            assert_eq!(result, Some(i));
        }
    }

    #[tokio::test]
    async fn test_multiple_normal_messages_preserve_order() {
        let (_priority_tx, priority_rx) = mpsc::unbounded_channel();
        let (normal_tx, normal_rx) = mpsc::unbounded_channel();
        let mut receiver = PriorityReceiver::new(priority_rx, normal_rx);

        // Send multiple normal messages
        for i in 1..=5 {
            normal_tx.send(i).unwrap();
        }

        // Should receive them in order
        for i in 1..=5 {
            let result = receiver.recv().await;
            assert_eq!(result, Some(i));
        }
    }
}

