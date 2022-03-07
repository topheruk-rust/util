use actix_web::{web::{ServiceConfig, self, resource}, Resource};
use handler::index;

mod handler;

pub fn app_config(config: &mut ServiceConfig) {
    config
        .service(
            resource("/hello")
            .route(web::get().to(index))
        );
}

#[cfg(test)]
mod tests {
    use actix_web::{
        http::{Method, StatusCode},
        test,
        web::Bytes,
        App,
    };

    use crate::app_config;

    trait BodyTest {
        fn as_str(&self) -> &str;
    }

    impl BodyTest for Bytes {
        fn as_str(&self) -> &str {
            std::str::from_utf8(self).unwrap()
        }
    }

    struct TestCase {
        path: &'static str,
        status: StatusCode,
        method: Method,
    }

    #[actix_web::test]
    async fn test_get_hello(){
        let app = test::init_service(App::new().configure(app_config)).await;
    
        let req =  test::TestRequest::get()
            .uri("/hello")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_get_error(){
        let app = test::init_service(App::new().configure(app_config)).await;
    
        let req =  test::TestRequest::get()
            .uri("/404")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_post_hello(){
        let app = test::init_service(App::new().configure(app_config)).await;
    
        let req =  test::TestRequest::post()
            .uri("/hello")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[actix_web::test]
    async fn test_app() {
        let app = test::init_service(App::new().configure(app_config)).await;

        let tt = vec![
            TestCase {
                path: "/hello",
                status: StatusCode::OK,
                method: Method::GET,
            },
            TestCase {
                path: "/404",
                status: StatusCode::NOT_FOUND,
                method: Method::GET,
            },
            TestCase {
                path: "/hello",
                status: StatusCode::METHOD_NOT_ALLOWED,
                method: Method::POST,
            },
        ];

        for tc in tt {
            let req = 
            // test::TestRequest::get()
                 match tc.method {
                    Method::POST => test::TestRequest::post(),
                    _ => test::TestRequest::default(),
                }
                .uri(tc.path)
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), tc.status);
        }
    }
}
