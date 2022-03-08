mod error {
    use crate::model;

    #[derive(Debug)]
    pub enum Error {
        Model(model::error::Error),
    }

    impl From<model::error::Error> for Error {
        fn from(e: model::error::Error) -> Self {
            Self::Model(e)
        }
    }
}

mod model {
    pub mod error {
        use super::*;

        #[derive(Debug)]
        pub enum Error {
            Repo(repo::error::Error),
            User(user::error::Error),
        }

        impl From<user::error::Error> for Error {
            fn from(e: user::error::Error) -> Self {
                Self::User(e)
            }
        }

        impl From<repo::error::Error> for Error {
            fn from(e: repo::error::Error) -> Self {
                Self::Repo(e)
            }
        }
    }

    pub mod repo {
        pub mod error {

            #[derive(Debug)]
            pub enum Error {
                NotFound,
                Invalid,
            }
        }

        use axum::async_trait;
        use uuid::Uuid;
        #[async_trait]
        pub trait Repo<T, U> {
            fn find(&self, id: Uuid) -> Result<T, error::Error>;
            fn push(&mut self, dto: U) -> Result<Uuid, error::Error>;
        }
    }

    pub mod user {
        use axum::async_trait;

        pub mod error {
            use crate::model::repo;

            #[derive(Debug)]
            pub enum Error {
                Encoding(serde_json::error::Error),
                Repo(repo::error::Error),
            }

            impl From<serde_json::error::Error> for Error {
                fn from(e: serde_json::error::Error) -> Self {
                    Error::Encoding(e)
                }
            }

            impl From<repo::error::Error> for Error {
                fn from(e: repo::error::Error) -> Self {
                    Error::Repo(e)
                }
            }
        }

        pub struct UserRepo {
            db: Vec<User>,
        }

        impl UserRepo {
            pub fn new() -> Self {
                Self { db: vec![] }
            }
        }

        #[async_trait]
        impl repo::Repo<User, UserDto> for UserRepo {
            fn find(&self, id: Uuid) -> Result<User, repo::error::Error> {
                match self.db.iter().find(|&u| u.id == id) {
                    Some(u) => Ok(u.clone()),
                    None => Err(repo::error::Error::NotFound),
                }
            }
            fn push(&mut self, dto: UserDto) -> Result<Uuid, repo::error::Error> {
                let u: User = dto.into(); // try_into else Invalid
                self.db.push(u.clone());
                Ok(u.id)
            }
        }

        use serde::Deserialize;
        use uuid::Uuid;

        use super::repo;

        #[derive(Debug, Clone)]
        pub struct User {
            pub id: Uuid,
            pub name: String,
        }

        impl User {
            pub fn new(UserDto { name }: UserDto) -> Self {
                Self {
                    id: Uuid::new_v4(),
                    name,
                }
            }
            pub fn default() -> Self {
                Self {
                    id: Uuid::default(),
                    name: String::default(),
                }
            }
        }

        impl From<UserDto> for User {
            fn from(dto: UserDto) -> Self {
                Self::new(dto)
            }
        }

        #[derive(Debug, Deserialize)]
        pub struct UserDto {
            pub name: String,
        }

        #[cfg(test)]
        mod test {
            use crate::model::repo::Repo;

            use super::*;

            type Result<T> = std::result::Result<T, error::Error>;

            macro_rules! create_user {
                ($name:ident, $inp:expr, $res:expr) => {
                    #[test]
                    fn $name() -> Result<()> {
                        let u = serde_json::from_str::<UserDto>($inp)?;
                        assert_eq!(u.name, $res.to_string());

                        let u: User = u.into();

                        Ok(println!("{}", u.id))
                    }
                };
            }

            create_user!(john, r#"{"name":"John"}"#, "John");

            #[test]
            fn test_repo() -> Result<()> {
                let mut repo = UserRepo::new();

                let u = serde_json::from_str::<UserDto>(r#"{"name":"John"}"#)?;
                repo.push(u)?;

                let u = serde_json::from_str::<UserDto>(r#"{"name":"Mary"}"#)?;
                let id = repo.push(u)?;

                let u = serde_json::from_str::<UserDto>(r#"{"name":"Tony"}"#)?;
                repo.push(u)?;

                assert_eq!(repo.db.len(), 3);

                let v = repo.find(id)?;
                assert_eq!(v.name, "Mary".to_string());

                Ok(())
            }
        }
    }
}

mod handler {}

fn main() {
    println!("Hello, world!");
}
