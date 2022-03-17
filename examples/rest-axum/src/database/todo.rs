use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use axum::async_trait;
use uuid::Uuid;

use crate::{
    database::{Error, Repo, Result},
    model::todo::{Todo, TodoCreate, TodoUpdate},
};

#[derive(Debug, Clone)]
pub struct TodoRepo {
    db: Arc<RwLock<HashMap<Uuid, Todo>>>,
}

impl TodoRepo {
    pub fn default() -> Self {
        Self { db: Arc::default() }
    }
}

#[async_trait]
impl Repo<Todo, TodoCreate, TodoUpdate> for TodoRepo {
    async fn find(&self, id: Uuid) -> Result<Todo> {
        let todos = self.db.read().unwrap(); // FIXME: say no to unwrap!!
        let todo = todos.get(&id);

        match todo {
            Some(todo) => Ok(todo.clone()),
            None => Err(Error::NotFound),
        }
    }

    async fn find_all(&self) -> Result<Vec<Todo>> {
        let todos = self.db.read().unwrap(); // FIXME: say no to unwrap!!
        let todos = todos.values().cloned().collect::<Vec<_>>();

        Ok(todos)
    }

    async fn create(&mut self, dto: TodoCreate) -> Result<Uuid> {
        let todo: Todo = dto.try_into()?;
        let id = todo.id;

        let mut todos = self.db.write().unwrap(); // FIXME: say no to unwrap!!
        todos.insert(id, todo);

        Ok(id)
    }

    async fn delete(&mut self, id: Uuid) -> Result<()> {
        let mut todos = self.db.write().unwrap();

        match todos.remove(&id).is_some() {
            true => Ok(()),
            false => Err(Error::NotFound),
        }
    }

    async fn update(&mut self, id: Uuid, dto: TodoUpdate) -> Result<Todo> {
        let mut todo = self.find(id).await?;

        if let Some(text) = dto.text {
            todo.text = text;
        }

        if let Some(completed) = dto.completed {
            todo.completed = completed;
        }

        let mut todos = self.db.write().unwrap(); // FIXME: say no to unwrap!!

        todos.insert(todo.id, todo.clone());

        Ok(todo)
    }
}
