use crate::groups::UserGroups;
use crate::http_responses::HttpResponseExt;
use crate::{auth::*, models::*, schema::*};
use actix_multipart::Multipart;
use actix_web::http::header;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::result::DatabaseErrorKind;
use futures_util::{StreamExt, TryStreamExt};
use serde::Deserialize;
use serde_json::json;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::Write;
use std::time::SystemTime;
use uuid::{Timestamp, Uuid};

type DbPool = Pool<ConnectionManager<SqliteConnection>>;

#[derive(Deserialize)]
pub struct LoginData {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterData {
    username: String,
    password: String,
    group: String,
}

#[derive(Deserialize)]
pub struct ChangePassword {
    password: String,
}

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login)))
        .service(web::resource("/register").route(web::post().to(register)))
        .service(web::resource("/change-password").route(web::post().to(change_password)));
}

pub fn file_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/upload").route(web::post().to(upload)))
        .service(web::resource("/download/{filename}").route(web::get().to(download_file)));
}

async fn login(pool: web::Data<DbPool>, info: web::Json<LoginData>) -> impl Responder {
    use crate::schema::users::dsl::*;
    let conn = &mut pool.get().unwrap();

    let user: User = match users.filter(username.eq(&info.username)).first(conn) {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::Unauthorized().unauthorized_default_body();
        }
    };

    if bcrypt::verify(&info.password, &user.password).unwrap() {
        let token = create_jwt(&user.username, &user.group_name);
        HttpResponse::Ok().json(json!(
            {"token": token }
        ))
    } else {
        HttpResponse::Unauthorized().unauthorized_default_body()
    }
}

async fn register(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    info: web::Json<RegisterData>,
) -> impl Responder {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .replace("Bearer ", "");

    if token.is_empty() {
        return HttpResponse::Unauthorized().unauthorized_default_body();
    }

    if let Some(claims) = verify_jwt(&token) {
        if claims.group != UserGroups::Admin.to_string() {
            return HttpResponse::Forbidden().forbbiden_default_body();
        }

        if !UserGroups::is_valid(&info.group) {
            return HttpResponse::BadRequest().bad_request_default_body("Invalid group");
        }

        let hashed = bcrypt::hash(&info.password, bcrypt::DEFAULT_COST).unwrap();
        let new_user = NewUser {
            username: info.username.clone(),
            password: hashed,
            group_name: info.group.clone(),
        };

        let conn = &mut pool.get().unwrap();
        let database_response = diesel::insert_into(users::table)
            .values(&new_user)
            .execute(conn);

        match database_response {
            Ok(_) => (),
            Err(
                e @ diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _),
            ) => {
                println!("{:?}", e);
                return HttpResponse::Conflict().conflict_default_body("User already exists");
            }
            Err(e) => {
                println!("{:?}", e);
                return HttpResponse::InternalServerError().internal_server_error_default_body();
            }
        }

        HttpResponse::Ok().json(json!({
            "username": info.username,
            "group": info.group,
        }))
    } else {
        HttpResponse::Unauthorized().unauthorized_default_body()
    }
}

async fn change_password(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    data: web::Json<ChangePassword>,
) -> impl Responder {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .replace("Bearer ", "");

    if token.is_empty() {
        return HttpResponse::Unauthorized().unauthorized_default_body();
    }

    if let Some(claims) = verify_jwt(&token) {
        let conn = &mut pool.get().unwrap();
        let hashed = bcrypt::hash(&data.password, bcrypt::DEFAULT_COST).unwrap();
        diesel::update(users::dsl::users.filter(users::dsl::username.eq(claims.sub)))
            .set(users::dsl::password.eq(hashed))
            .execute(conn)
            .unwrap();
        HttpResponse::Ok().body(json!({"message": "Senha alterada com sucesso"}).to_string())
    } else {
        HttpResponse::Unauthorized().unauthorized_default_body()
    }
}

async fn upload(mut payload: Multipart) -> impl Responder {
    println!("Starting Upload");
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let filename = if let Some(fname) = content_disposition.unwrap().get_filename() {
            fname.to_string()
        } else {
            format!("upload-{}", Uuid::new_v4())
        };

        let filepath = format!("./uploads/{}_{}", uuid::Uuid::new_v4(), filename);
        println!("Filename: {}", filename);

        // Cria o arquivo localmente
        let mut f = match File::create(&filepath) {
            Ok(file) => file,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(format!("Erro ao criar arquivo: {}", e));
            }
        };
        println!("Arquivo Criado");

        // Escreve os chunks no arquivo
        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(data) => data,
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .body(format!("Erro no chunk: {}", e));
                }
            };
            if let Err(e) = f.write_all(&data) {
                return HttpResponse::InternalServerError()
                    .body(format!("Erro ao escrever arquivo: {}", e));
            }
        }
        println!("Arquivo escrito");
    }
    println!("==================");
    HttpResponse::Ok().body("Arquivo enviado com sucesso!")
}

async fn download_file(req: HttpRequest, path: web::Path<String>) -> impl Responder {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .replace("Bearer ", "");
    if token.is_empty() {
        return HttpResponse::Unauthorized().unauthorized_default_body();
    }
    if verify_jwt(&token).is_none() {
        return HttpResponse::Unauthorized().unauthorized_default_body();
    }

    let filename = path.into_inner();
    let filepath = format!("uploads/{}", filename);
    if std::path::Path::new(&filepath).exists() {
        actix_files::NamedFile::open_async(filepath)
            .await
            .unwrap()
            .into_response(&req)
    } else {
        HttpResponse::NotFound().not_found_default_body("Arquivo não encontrado")
    }
}
