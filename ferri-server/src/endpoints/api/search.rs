use rocket::{
    get, serde::json::Json, FromFormField, State,
};
use main::types::{api, get};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::{http_wrapper::HttpWrapper, AuthenticatedUser, Db};

#[derive(Serialize, Deserialize, FromFormField, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SearchType {
    Accounts,
    Hashtags,
    Statuses,
    All
}

impl Default for SearchType {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Serialize, Deserialize)]
pub struct SearchResults {
    statuses: Vec<api::Status>,
    accounts: Vec<api::Account>,
    hashtags: Vec<()>
}

#[get("/search?<q>&<type>")]
pub async fn search(
    q: &str,
    r#type: SearchType,
    helpers: &State<crate::Helpers>,
    mut db: Connection<Db>,
    user: AuthenticatedUser
) -> Json<SearchResults> {
    let ty = r#type;
    info!("search for {} (ty: {:?})", q, ty);
    
    let key_id = "https://ferri.amy.mov/users/9b9d497b-2731-435f-a929-e609ca69dac9#main-key";
    let http = HttpWrapper::new(&helpers.http, key_id);
    
    let mut accounts = vec![];
    let mut statuses = vec![];
    
    match ty {
        SearchType::Accounts => {
            let person = {
                let res = http.get_person(q).await;
                if let Err(e) = res {
                    error!("could not load user {}: {}", q, e.to_string());
                    None
                } else {
                    Some(res.unwrap())
                }
            };

            let user = get::user_by_actor_uri(person.unwrap().obj.id, &mut db)
                .await
                .unwrap();

            accounts.push(user.into())
        },
        SearchType::Statuses => {
            if q == "me" {
                let st = get::posts_for_user_id(user.id, &mut db)
                    .await
                    .unwrap();

                for status in st.into_iter() {
                    statuses.push(status.into());
                }
            }
        },
        SearchType::Hashtags => todo!(),
        SearchType::All => todo!(),
    }

    Json(SearchResults {
        statuses,
        accounts,
        hashtags: vec![],
    })
}
