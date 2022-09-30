use std::any::Any;

use rtic_actor_traits::Post;

/// An implementation of `Post` that accepts "any" message type and lets you inspect all `post`-ed
/// messages
#[derive(Default)]
pub struct PostSpy {
    posted_messages: Vec<Box<dyn Any>>,
}

impl PostSpy {
    /// Returns an *iterator* over the posted messages
    ///
    /// Note that you must specify *which* type of message you want to retrieve (the `T` in the
    /// signature)
    /// In practice, this will most likely mean using "turbo fish" syntax to specify the type:
    /// `post_spy.posted_messages::<MyMessage>()`
    pub fn posted_messages<T>(&self) -> impl Iterator<Item = &T>
    where
        T: Any,
    {
        self.posted_messages
            .iter()
            .filter_map(|message| message.downcast_ref())
    }
}

impl<M> Post<M> for PostSpy
where
    M: Any,
{
    fn post(&mut self, message: M) -> Result<(), M> {
        self.posted_messages.push(Box::new(message));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_and_inspect() {
        let mut spy = PostSpy::default();
        assert_eq!(None, spy.posted_messages::<i32>().next());
        spy.post(42).unwrap();
        assert_eq!(vec![&42], spy.posted_messages::<i32>().collect::<Vec<_>>());
    }

    #[test]
    fn can_post_two_types_to_the_same_spy() {
        #[derive(Debug, PartialEq)]
        struct MessageA(i32);
        #[derive(Debug, PartialEq)]
        struct MessageB(i32);

        let mut post_spy = PostSpy::default();
        post_spy.post(MessageA(0)).unwrap();
        post_spy.post(MessageB(1)).unwrap();
        post_spy.post(MessageA(2)).unwrap();
        post_spy.post(MessageB(3)).unwrap();

        // peek *only* `MessageA` messages in `post` order
        assert_eq!(
            vec![&MessageA(0), &MessageA(2)],
            post_spy.posted_messages::<MessageA>().collect::<Vec<_>>()
        );

        // peek *only* `MessageB` messages in `post` order
        assert_eq!(
            vec![&MessageB(1), &MessageB(3)],
            post_spy.posted_messages::<MessageB>().collect::<Vec<_>>()
        );
    }
}
