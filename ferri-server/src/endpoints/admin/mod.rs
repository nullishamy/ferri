use rocket::{get, post, response::content::RawHtml};
use askama::Template; 

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    val: String
}

#[post("/clicked")]
pub async fn button_clicked() -> RawHtml<String> {
    let tmpl = IndexTemplate { val: "clicked".to_string() };
    RawHtml(tmpl.render().unwrap())
}

#[get("/")]
pub async fn index() -> RawHtml<String> {
    let tmpl = IndexTemplate { val: "test".to_string() };
    RawHtml(tmpl.render().unwrap())
}
