use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::error::Error;
use crate::model::Result;

#[derive(Debug, Serialize, Clone)]
pub struct Todo {
    pub id: Uuid,
    pub text: String,
    pub completed: bool,
}

impl Todo {
    // TODO:  what if empty string
    // non lic should prevent this being an issue
    fn new(text: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            text,
            completed: false,
        }
    }
}

impl TryFrom<TodoCreate> for Todo {
    type Error = Error;

    fn try_from(value: TodoCreate) -> Result<Self> {
        let TodoCreate { text } = match value {
            _ if value.text.is_empty() => Err(Error::EmptyText),
            _ => Ok(value),
        }?;

        Ok(Todo::new(text))
    }
}

#[derive(Debug, Deserialize)]
pub struct TodoUpdate {
    pub text: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TodoCreate {
    pub text: String,
}
