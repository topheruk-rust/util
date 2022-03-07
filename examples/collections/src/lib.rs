use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
struct User {
    name: String,
}

type Database = Arc<RwLock<HashMap<u32, User>>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_insert() {
        let db = Database::default();

        // -- insert
        db.write().unwrap().insert(
            1,
            User {
                name: "Foo".to_string(),
            },
        );

        // -- find
        if let Some(a) = db.read().unwrap().get(&1) {
            assert_eq!(a.name, "Foo".to_string());
        };
    }
}
