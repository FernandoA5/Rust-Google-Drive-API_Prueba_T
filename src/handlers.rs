use actix_web::{get, post, HttpResponse, Responder};
use actix_web::{web::Data, web::Query};
use actix_multipart::Multipart;
use futures_util::StreamExt as _;
use futures_util::TryStreamExt;
use std::collections::HashMap;

use crate::state::AppState;
use serde::Serialize;


// Estructura personalizada para los archivos
#[derive(Serialize)]
struct FileInfo {
    id: Option<String>,
    name: Option<String>,
    mime_type: Option<String>,
    created_time: Option<String>,
}

#[get("/drive/files")]
pub async fn list_files(data: Data<AppState>, folder_id: Query<HashMap<String, String>>) -> impl Responder {
    let hub = data.hub.lock().await;

    // Obtenemos el valor de 'folder_id' del mapa de parámetros
    let folder_id = match folder_id.get("folder_id") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().body("El parámetro 'folder_id' es requerido.");
        }
    };

    // Query para filtrar los archivos por el folder_id proporcionado
    let query = format!("'{}' in parents", folder_id);

    // Especificamos los campos que queremos recuperar
    let fields = "files(id,name,mimeType,createdTime)";

    match hub.files().list().q(&query).add_scope("https://www.googleapis.com/auth/drive.readonly").param("fields", fields).doit().await {
        Ok((_, file_list)) => {
            // Extraemos solo la información necesaria
            let files: Vec<FileInfo> = file_list.files.unwrap_or_default().into_iter().map(|file| {
                FileInfo {
                    id: file.id,
                    name: file.name,
                    mime_type: file.mime_type,
                    created_time: file.created_time,
                }
            }).collect();

            // Devolvemos la lista de archivos filtrada
            HttpResponse::Ok().json(files)
        }
        Err(e) => {
            eprintln!("Error al listar archivos: {:?}", e);
            HttpResponse::InternalServerError().body("Error al listar archivos")
        }
    }
}



#[post("drive/files/upload")]
pub async fn upload_file(
    mut payload: Multipart,
    data: Data<AppState>,
    folder_id: Query<HashMap<String, String>>, // Recibimos el ID de la carpeta como parámetro
) -> impl Responder {
    let mut file_content = Vec::new();
    let mut file_name = "uploaded.pdf".to_string();  // Nombre por defecto

    // Procesamos el archivo que viene en la solicitud
    while let Ok(Some(mut field)) = payload.try_next().await {
        let disposition = field.content_disposition();

        // Obtenemos el nombre original del archivo si está presente
        if let Some(filename) = disposition.get_filename() {
            file_name = filename.to_string();
        }

        // Leemos el contenido del archivo
        while let Some(chunk) = field.next().await {
            match chunk {
                Ok(data) => file_content.extend_from_slice(&data),
                Err(e) => {
                    eprintln!("Error al leer el archivo: {:?}", e);
                    return HttpResponse::InternalServerError().body("Error al leer el archivo");
                }
            }
        }
    }

    // Nos aseguramos de que el contenido no esté vacío
    if file_content.is_empty() {
        return HttpResponse::BadRequest().body("El archivo está vacío.");
    }

    let hub = data.hub.lock().await;

    // Obtenemos el folder_id de la URL o body de la solicitud
    let folder_id = match folder_id.get("folder_id") {
        Some(id) => id.to_string(),
        None => return HttpResponse::BadRequest().body("El parámetro 'folder_id' es requerido."),
    };

    // Subimos el archivo a Google Drive con el nombre original y en la carpeta especificada
    let result = hub
        .files()
        .create(google_drive3::api::File {
            name: Some(file_name.clone()),  // Usamos el nombre original del archivo
            mime_type: Some("application/pdf".to_string()),
            parents: Some(vec![folder_id]), // Especificamos el folder_id en 'parents'
            ..Default::default()
        })
        .upload(
            std::io::Cursor::new(file_content), // El contenido del archivo
            "application/pdf".parse().unwrap(),
        )
        .await;

    match result {
        Ok((_, file)) => HttpResponse::Ok().json(file),
        Err(e) => {
            eprintln!("Error al subir archivo: {:?}", e);
            HttpResponse::InternalServerError().body("Error al subir archivo")
        }
    }
}



#[get("/drive/files/{file_id}")]
pub async fn download_file(
    data: Data<AppState>,
    path: actix_web::web::Path<String>,
) -> impl Responder {
    let file_id = path.into_inner();
    let hub = data.hub.lock().await;

    match hub
        .files()
        .get(&file_id)
        .param("alt", "media")
        .add_scope("https://www.googleapis.com/auth/drive.readonly")
        .doit()
        .await
    {
        Ok((response, _)) => {
            // Utilizamos hyper::body::to_bytes para obtener el contenido del cuerpo
            let bytes = hyper::body::to_bytes(response.into_body()).await;
            match bytes {
                Ok(content) => {
                    HttpResponse::Ok()
                        .content_type("application/pdf")
                        .body(content)
                }
                Err(e) => {
                    eprintln!("Error al leer el archivo: {:?}", e);
                    HttpResponse::InternalServerError().body("Error al leer el archivo")
                }
            }
        }
        Err(e) => {
            eprintln!("Error al descargar archivo: {:?}", e);
            HttpResponse::InternalServerError().body("Error al descargar archivo")
        }
    }
}