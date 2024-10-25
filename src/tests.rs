#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use actix_web::web::Data;
    use crate::handlers::{list_files, upload_file, download_file};
    use crate::state::AppState;
    use google_drive3::DriveHub;
    use tokio::sync::Mutex;
    use hyper::Client;
    use hyper_rustls::HttpsConnector;
    use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod, ApplicationSecret};
    use std::default::Default;

    #[actix_web::test]
    async fn test_list_files() {
        // Configurar el cliente HTTPS
        let https_connector = HttpsConnector::with_native_roots();
        let client = Client::builder().build::<_, hyper::Body>(https_connector);

        // Crear un autenticador simulado
        let secret: ApplicationSecret = Default::default();
        let auth = InstalledFlowAuthenticator::builder(
            secret,
            InstalledFlowReturnMethod::HTTPRedirect,
        )
        .build()
        .await
        .unwrap();

        let hub = DriveHub::new(client, auth);

        let app_state = Data::new(AppState {
            hub: Mutex::new(hub),
        });

        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .service(list_files),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/drive/files?folder_id=123")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_upload_file() {
        // Configurar el cliente HTTPS
        let https_connector = HttpsConnector::with_native_roots();
        let client = Client::builder().build::<_, hyper::Body>(https_connector);

        // Crear un autenticador simulado
        let secret: ApplicationSecret = Default::default();
        let auth = InstalledFlowAuthenticator::builder(
            secret,
            InstalledFlowReturnMethod::HTTPRedirect,
        )
        .build()
        .await
        .unwrap();

        let hub = DriveHub::new(client, auth);

        let app_state = Data::new(AppState {
            hub: Mutex::new(hub),
        });

        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .service(upload_file),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/drive/files/upload?folder_id=123")
            .set_payload(
                "--boundary\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.pdf\"\r\nContent-Type: application/pdf\r\n\r\nTest content\r\n--boundary--\r\n",
            )
            .insert_header(("Content-Type", "multipart/form-data; boundary=boundary"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_download_file() {
        // Configurar el cliente HTTPS
        let https_connector = HttpsConnector::with_native_roots();
        let client = Client::builder().build::<_, hyper::Body>(https_connector);

        // Crear un autenticador simulado
        let secret: ApplicationSecret = Default::default();
        let auth = InstalledFlowAuthenticator::builder(
            secret,
            InstalledFlowReturnMethod::HTTPRedirect,
        )
        .build()
        .await
        .unwrap();

        let hub = DriveHub::new(client, auth);

        let app_state = Data::new(AppState {
            hub: Mutex::new(hub),
        });

        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .service(download_file),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/drive/files/123")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
