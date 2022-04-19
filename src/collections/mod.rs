pub mod list;

pub mod list_ {
    type Link<T> = Option<Box<Node<T>>>;

    pub struct List<T> {
        head: Link<T>,
    }

    impl<T> List<T> {
        fn new() -> Self {
            Self::default()
        }

        pub fn push(&mut self, elem: T) {
            self.head = Some(Box::new(Node {
                elem,
                next: self.head.take(),
            }));
        }

        pub fn pop(&mut self) -> Option<T> {
            self.head.take().map(|node| {
                self.head = node.next;
                node.elem
            })
        }

        pub fn peek(&mut self) -> Option<&T> {
            self.head.as_ref().map(|node| &node.elem)
        }
    }

    impl<T> Default for List<T> {
        fn default() -> Self {
            Self { head: None }
        }
    }

    struct Node<T> {
        elem: T,
        next: Link<T>,
    }

    impl<T> Node<T> {
        fn new(elem: T, next: Link<T>) -> Box<Self> {
            Box::new(Node { elem, next: None })
        }

        fn set_next(&mut self, next: Node<T>) {
            self.next = Some(Box::new(next))
        }
    }

    mod test {
        use super::*;

        #[test]
        fn push() {
            let mut list = List::default();
            list.push(1); //1
            list.push(2); //2->1
            list.push(3); //3->2->1

            let value = list.pop(); //3
            let value = list.pop(); //2

            assert_eq!(value, Some(2))
        }
    }
}

mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
