#[macro_use]
extern crate rocket;

use handler::*;
use rocket::{routes, Build};

mod handler;
mod middleware;

pub fn rocket() -> rocket::Rocket<Build> {
    rocket::build().mount("/", routes![world])
}

#[cfg(test)]
mod tests {
    use super::rocket;
    use rocket::{
        http::{Method, Status},
        local::blocking::Client,
    };

    struct TestCase {
        path: &'static str,
        status: Status,
        method: Method,
    }

    #[test]
    fn test_get_world() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");

        let response = client.get("/world").dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn test_get_error() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");

        let response = client.get("/404").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn test_post_world() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");

        let response = client.post("/world").dispatch();
        assert_eq!(response.status(), Status::MethodNotAllowed);
    }

    #[test]
    fn test_app() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");

        let tt = vec![
            TestCase {
                path: "/world",
                status: Status::Ok,
                method: Method::Get,
            },
            TestCase {
                path: "/world",
                status: Status::MethodNotAllowed,
                method: Method::Post,
            },
        ];

        for tc in tt {
            let response = match tc.method {
                Method::Post => client.post(tc.path),
                _ => client.get(tc.path),
            }
            .dispatch();

            // client.get(tc.path).dispatch();
            assert_eq!(response.status(), tc.status);
        }
    }
}
