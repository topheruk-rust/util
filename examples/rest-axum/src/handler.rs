use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    Json,
};
use bson::doc;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    database::{movie::MovieRepo, todo::TodoRepo, Repo},
    error::Result,
    model::{
        movie::Movie,
        todo::{Todo, TodoCreate, TodoUpdate},
    },
};

pub async fn todos_index(Extension(repo): Extension<TodoRepo>) -> Result<Json<Vec<Todo>>> {
    let todos = repo.find_all().await?;

    Ok(Json(todos))
}

pub async fn todos_member(
    Path(id): Path<Uuid>,
    Extension(repo): Extension<TodoRepo>,
) -> Result<Json<Todo>> {
    let todo = repo.find(id).await?;

    Ok(Json(todo))
}

pub async fn todos_create(
    Json(dto): Json<TodoCreate>,
    Extension(mut repo): Extension<TodoRepo>,
) -> Result<(StatusCode, Json<Value>)> {
    let uid = repo.create(dto).await?;

    Ok((StatusCode::CREATED, Json(json!({ "id": uid }))))
}

pub async fn todos_update(
    Path(id): Path<Uuid>,
    Json(dto): Json<TodoUpdate>,
    Extension(mut repo): Extension<TodoRepo>,
) -> Result<(StatusCode, Json<Todo>)> {
    let todo = repo.update(id, dto).await?;

    Ok((StatusCode::CREATED, Json(todo)))
}

pub async fn todos_delete(
    Path(id): Path<Uuid>,
    Extension(mut repo): Extension<TodoRepo>,
) -> Result<StatusCode> {
    repo.delete(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

// -- movies

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub offset: Option<u32>,
    pub limit: Option<u32>, // this will be too large
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            offset: Some(0),
            limit: Some(20),
        }
    }
}

pub async fn movies_index(
    pagination: Option<Query<Pagination>>,
    Extension(repo): Extension<MovieRepo>,
) -> Result<Json<Vec<Movie>>> {
    let Query(Pagination { offset, limit }) = pagination.unwrap_or_default();

    let skip = offset.unwrap_or(0);
    let stage_skip = doc! { "$skip": skip };

    // if number is greater than `MAX` then default
    let limit = limit.unwrap_or(20);
    let stage_limit = doc! { "$limit": limit };

    let todos = repo.find(vec![stage_skip, stage_limit]).await?;
    Ok(Json(todos))
}
