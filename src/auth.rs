use google_drive3::DriveHub;
use yup_oauth2::InstalledFlowAuthenticator;

pub async fn get_drive_hub() -> DriveHub {
    let secret = yup_oauth2::read_application_secret("credentials.json")
        .await
        .expect("No se pudo leer el archivo de credenciales");

    let auth = InstalledFlowAuthenticator::builder(
        secret,
        yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk("token_cache.json")
    .build()
    .await
    .expect("Error al construir el autenticador");

    let hub = DriveHub::new(
        hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
        auth,
    );

    hub
}
