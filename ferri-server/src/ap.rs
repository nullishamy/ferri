use rocket::serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Link {
    pub rel: String,
    #[serde(rename = "type")]
    pub ty: Option<String>,
    pub href: Option<String>
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct WebfingerResponse {
    pub subject: String,
    pub aliases: Vec<String>,
    pub links: Vec<Link>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(crate = "rocket::serde")]
pub struct UserKey {
    pub id: String,
    pub owner: String,
    #[serde(rename = "publicKeyPem")]
    pub public_key: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(crate = "rocket::serde")]
pub struct Person {
    // FIXME: This is because Masto sends an array but we don't care
    #[serde(rename = "@context")]
    #[serde(skip_deserializing)]
    pub context: String,
    
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub following: String,
    pub followers: String,
    pub inbox: String,
    pub outbox: String,
    pub preferred_username: String,
    pub name: String,
    pub summary: String,
    pub public_key: Option<UserKey>
    // pub url: String,
    // pub manually_approves_followers: bool,
    // pub discoverable: bool,
    // pub indexable: bool,
    // pub published: String,
    // pub memorial: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct MinimalActivity {
	pub id: String,
	#[serde(rename = "type")]
    pub ty: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct DeleteActivity  {
	pub id: String,
	#[serde(rename = "type")]
    pub ty: String,

	pub object: String,
	pub actor: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct CreateActivity  {
	pub id: String,
	#[serde(rename = "type")]
    pub ty: String,

	pub object: Post,
	pub actor: String,
	pub to: Vec<String>,
	pub cc: Vec<String>,
	#[serde(rename = "published")]
	pub ts: String,
	pub summary: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct FollowActivity  {
	pub id: String,
	#[serde(rename = "type")]
    pub ty: String,

	pub object: String,
	pub actor: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct AcceptActivity  {
	#[serde(rename = "type")]
    pub ty: String,

	pub object: String,
	pub actor: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Post {
	// FIXME: This is because Masto sends an array but we don't care
    #[serde(rename = "@context")]
    #[serde(skip_deserializing)]
    pub context: String,
	pub id: String,
	#[serde(rename = "type")]
    pub ty: String,
	#[serde(rename = "published")]
	pub ts: String,
	pub content: String,
	pub to: Vec<String>,
	pub cc: Vec<String>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Activity {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
    
    pub summary: String,
    pub actor: String,
    pub object: String,
    pub published: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(crate = "rocket::serde")]
pub struct Object {
	pub id: String,
    #[serde(rename = "type")]
    pub ty: String,
	pub object: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(crate = "rocket::serde")]
pub struct OrderedCollection {
    pub summary: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub total_items: u64,
    pub ordered_items: Vec<String>
}
