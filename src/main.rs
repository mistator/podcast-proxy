use std::{env, io};
use std::sync::{RwLock};
use actix_web::{get, error, Responder, HttpServer, App, HttpResponse, web, HttpRequest};
use actix_web::http::header;
use actix_web::http::header::ContentType;
use serde::Deserialize;
use url::Url;
use crate::grabbers::Grabber;
use crate::ivoox::Ivoox;
use crate::youtube::YouTube;

pub mod grabbers;
pub mod feed;
pub mod ivoox;
pub mod youtube;

struct AppState {
    youtube: RwLock<YouTube>,
}

struct Settings {
    proxy: bool,
    public_url: Url,
}

#[derive(Deserialize)]
struct Qry {
    q: Url,
    list: Option<String>,
}

#[get("/")]
async fn home(data: web::Data<AppState>, settings: web::Data<Settings>, mut query: web::Query<Qry>) -> actix_web::Result<impl Responder> {
    let list = query.list.take();
    let url = &mut query.q;

    if let Some(list) = list {
        url.query_pairs_mut().append_pair("list", list.as_str());
    }

    let feed = match url.domain() {
        Some("www.ivoox.com") => Ivoox
            .make_feed(url, &settings.public_url)
            .await
            .map_err(|err| error::ErrorBadRequest(err.to_string())),
        Some("www.youtube.com") => {
            let youtube = data.youtube.read().unwrap();
            youtube.make_feed(url, &settings.public_url)
                .await
                .map_err(|err| error::ErrorBadRequest(err.to_string()))
        },
        _ => Err(error::ErrorBadRequest("invalid url"))
    }?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::xml())
        .body(feed.to_xml().map_err(|err| error::ErrorBadRequest(err.to_string()))?
        )
    )
}

#[get("/item/{provider}/{id}")]
async fn get_item(
    _req: HttpRequest,
    settings: web::Data<Settings>,
    data: web::Data<AppState>,
    path: web::Path<(String, String)>
) -> actix_web::Result<impl Responder> {
    let provider = &path.0;
    let id = &path.1;

    if !settings.proxy {
        let url = match provider.as_str() {
            "ivoox" => Ivoox.get_item_url(&id)
                .await
                .map_err(|err| error::ErrorBadRequest(err.to_string()))?,
            "youtube" => {
                let youtube = data.youtube.read().unwrap();
                youtube.get_item_url(&id)
                    .await
                    .map_err(|err| error::ErrorBadRequest(err.to_string()))?
            },
            _ => return Err::<HttpResponse, _>(error::ErrorBadRequest("invalid url")),
        };

        Ok(HttpResponse::TemporaryRedirect()
            .append_header((header::LOCATION, url.as_str()))
            .finish())
    } else {
        todo!()
    }

}

/*
#[get("/yt/{id}")]
async fn yt(req: HttpRequest, data: web::Data<AppState>, id: web::Path<(String,)>) -> actix_web::Result<impl Responder> {
    let youtube = data.youtube.read().unwrap();
    let url = youtube.get_item_url(&id.0);

    let video_url = format!("https://www.youtube.com/watch?v={}", id.0);
    let filename = format!("{}.mp3", id.0);


    let out = {
        let youtube = data.youtube.read().unwrap();
        let output_file = youtube.output_dir().join(filename.clone());

        if true || !fs::exists(output_file.clone())? {
            let info = youtube.fetch_video_infos(video_url).await
                .map_err(|err| error::ErrorBadRequest(err.to_string()))?;

            let audio_format = info.best_audio_format().unwrap();
            let url = audio_format.url().unwrap();

            return Ok(HttpResponse::TemporaryRedirect()
                .append_header((header::LOCATION, url.as_str()))
                .finish());

            youtube.download(&info, filename)
                .audio_quality(AudioQuality::Best)
                .embed_thumbnail(true)
                .execute()
                .await
                .map_err(|err| error::ErrorBadRequest(err.to_string()))?;
        }

        output_file
    };

    let file = actix_files::NamedFile::open(out)?;
    Ok(file.into_response(&req))
}
*/

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    let public_url = env::var("PUBLIC_URL").unwrap_or("http://localhost:8080".to_string());
    let public_url = Url::parse(public_url.as_str())
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
    let proxy = env::var("PROXY").unwrap_or("false".to_string()) == "true";

    let state = web::Data::new(AppState {
        youtube: RwLock::new(YouTube::new().await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?)
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Settings {
                proxy,
                public_url: public_url.clone(),
            }))
            .app_data(state.clone())
            .service(home)
            .service(get_item)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

