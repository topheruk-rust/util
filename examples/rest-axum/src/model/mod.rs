pub mod error;
pub mod movie;
pub mod todo;

use error::Error;
type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod test {
    use crate::model::{
        todo::{Todo, TodoCreate},
        Result,
    };

    macro_rules! test_todo_create {
        ($name:ident, $str:expr) => {
            #[test]
            fn $name() {
                let t = TodoCreate {
                    text: $str.to_string(),
                };

                match t.try_into() as Result<Todo> {
                    Ok(t) => assert_eq!(t.completed, false),
                    Err(_) => assert!(true), // Err(e) => assert_eq!(e, error::Error::EmptyText),
                };
            }
        };
    }

    test_todo_create!(todo_create_todo_ok, "John");
    test_todo_create!(test_create_todo_err, "");

    #[test]
    fn test_todo_json() -> Result<()> {
        let t = serde_json::from_str::<TodoCreate>(r#"{"text":"Hello, World"}"#)?;
        let t: Todo = t.try_into()?;
        assert!(!t.completed);
        Ok(())
    }
}
