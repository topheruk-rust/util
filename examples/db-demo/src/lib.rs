use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

type Items = HashMap<String, i32>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Item {
    pub name: String,
    pub quantity: i32,
}

#[derive(Clone)]
pub struct Store {
    pub grocery_list: Arc<RwLock<Items>>,
}

impl Store {
    fn new() -> Self {
        Store {
            grocery_list: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}
