use rocket::{get, response::content::RawHtml};
use askama::Template; 

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {

}

#[get("/")]
pub async fn index() -> RawHtml<String> {
    let tmpl = IndexTemplate { };
    RawHtml(tmpl.render().unwrap())
}
