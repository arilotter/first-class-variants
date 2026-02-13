mod generated {
    use first_class_variants::first_class_variants;
    #[first_class_variants(derive(PartialEq, Eq, Copy, Clone))]
    #[derive(Debug)]
    pub enum Foo {
        #[derive(Debug)]
        Bar(u8),
        #[derive(Debug)]
        Spam { ham: u16, eggs: u32 },
    }
}

mod with_module {
    use first_class_variants::first_class_variants;
    #[first_class_variants(module = "variants", derive(PartialEq, Eq, Copy, Clone))]
    #[derive(Debug)]
    pub enum Baz {
        #[derive(Debug)]
        Qux(u8),
        #[derive(Debug)]
        Corge { grault: u16, garply: u32 },
    }
}

mod with_parent {
    use first_class_variants::first_class_variants;

    #[first_class_variants(
        module = "client",
        derive(Debug, PartialEq, Eq, Clone),
        impl_into_parent = "EventData"
    )]
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub enum Client {
        StateChanged { old_state: String, new_state: String },
        ErrorOccurred { message: String },
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub enum EventData {
        Client(Client),
    }
}

mod with_parent_no_module {
    use first_class_variants::first_class_variants;

    #[first_class_variants(
        derive(Debug, PartialEq, Eq, Clone),
        impl_into_parent = "Message::User"
    )]
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub enum UserEvent {
        LoggedIn { user_id: String },
        LoggedOut,
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub enum Message {
        User(UserEvent),
    }
}
mod tests {
    use crate::generated::*;
    use std::convert::TryInto;
    #[test]
    fn works() {
        let bar = FooBar(1);
        let spam = FooSpam { ham: 2, eggs: 3 };

        let bar_foo: Foo = bar.into();
        match bar_foo {
            Foo::Bar(x) => assert_eq!(x, bar),
            _ => unreachable!("bar_foo isn't a Foo::Bar"),
        }
        let spam_foo: Foo = spam.into();
        match spam_foo {
            Foo::Spam(x) => assert_eq!(x, spam),
            _ => unreachable!("spam_foo isn't a Foo::Spam"),
        }

        let maybe_bar: Result<FooBar, ()> = bar_foo.try_into();
        assert_eq!(maybe_bar, Ok(bar));

        // A useful pattern for pulling things out of a Foo
        assert!(if let Ok(FooBar(x)) = maybe_bar {
            println!("{}", x);
            true
        } else {
            false
        });

        // or, do it by ref!
        assert!(if let &Ok(FooBar(x)) = &Foo::Bar(FooBar(123)).try_into() {
            println!("{}", x);
            true
        } else {
            false
        });
    }

    #[test]
    fn works_with_module() {
        use crate::with_module::{variants::*, Baz};

        let qux = Qux(1);
        let corge = Corge {
            grault: 2,
            garply: 3,
        };

        let qux_baz: Baz = qux.into();
        match qux_baz {
            Baz::Qux(x) => assert_eq!(x, qux),
            _ => unreachable!("qux_baz isn't a Baz::Qux"),
        }
        let corge_baz: Baz = corge.into();
        match corge_baz {
            Baz::Corge(x) => assert_eq!(x, corge),
            _ => unreachable!("corge_baz isn't a Baz::Corge"),
        }

        let maybe_qux: Result<Qux, ()> = qux_baz.try_into();
        assert_eq!(maybe_qux, Ok(qux));

        assert!(if let Ok(Qux(x)) = maybe_qux {
            println!("{}", x);
            true
        } else {
            false
        });

        assert!(if let &Ok(Qux(x)) = &Baz::Qux(Qux(123)).try_into() {
            println!("{}", x);
            true
        } else {
            false
        });
    }

    #[test]
    fn works_with_parent() {
        use crate::with_parent::{client::*, Client, EventData};

        let state_changed = StateChanged {
            old_state: "running".to_string(),
            new_state: "stopped".to_string(),
        };
        let event: EventData = state_changed.clone().into();

        match event {
            EventData::Client(Client::StateChanged(s)) => {
                assert_eq!(s.old_state, "running");
                assert_eq!(s.new_state, "stopped");
            }
            _ => unreachable!("event isn't an EventData::Client::StateChanged"),
        }

        let error = ErrorOccurred {
            message: "connection failed".to_string(),
        };
        let event: EventData = error.into();

        match event {
            EventData::Client(Client::ErrorOccurred(e)) => {
                assert_eq!(e.message, "connection failed");
            }
            _ => unreachable!("event isn't an EventData::Client::ErrorOccurred"),
        }
    }

    #[test]
    fn works_with_parent_no_module() {
        use crate::with_parent_no_module::{Message, UserEvent, UserEventLoggedIn, UserEventLoggedOut};

        let logged_in = UserEventLoggedIn {
            user_id: "alice".to_string(),
        };
        let msg: Message = logged_in.into();

        match msg {
            Message::User(UserEvent::LoggedIn(u)) => {
                assert_eq!(u.user_id, "alice");
            }
            _ => unreachable!("msg isn't a Message::User::LoggedIn"),
        }

        let logged_out = UserEventLoggedOut;
        let msg: Message = logged_out.into();

        match msg {
            Message::User(UserEvent::LoggedOut(_)) => {},
            _ => unreachable!("msg isn't a Message::User::LoggedOut"),
        }
    }
}
