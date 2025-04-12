use rocket::{get, serde::json::Json};

use crate::types::instance::{
    Accounts, Configuration, Contact, Instance, MediaAttachments, Polls, Registrations, Statuses,
    Thumbnail, Translation, Urls,
};

#[get("/instance")]
pub async fn instance() -> Json<Instance> {
    Json(Instance {
        domain: "ferri.amy.mov".to_string(),
        title: "Ferri".to_string(),
        version: "0.0.1".to_string(),
        source_url: "https://forge.amy.mov/amy/ferri".to_string(),
        description: "ferriverse".to_string(),
        thumbnail: Thumbnail {
            url: "".to_string(),
        },
        icon: vec![],
        languages: vec![],
        configuration: Configuration {
            urls: Urls {
                streaming: "".to_string(),
                about: "".to_string(),
                privacy_policy: "".to_string(),
                terms_of_service: "".to_string(),
            },
            accounts: Accounts {
                max_featured_tags: 10,
                max_pinned_statuses: 10,
            },
            statuses: Statuses {
                max_characters: 1000,
                max_media_attachments: 5,
                characters_reserved_per_url: 10,
            },
            media_attachments: MediaAttachments {
                supported_mime_types: vec![],
                description_limit: 10,
                image_size_limit: 10,
                image_matrix_limit: 10,
                video_size_limit: 10,
                video_frame_rate_limit: 10,
                video_matrix_limit: 10,
            },
            polls: Polls {
                max_options: 10,
                max_characters_per_option: 10,
                min_expiration: 10,
                max_expiration: 10,
            },
            translation: Translation { enabled: false },
        },
        registrations: Registrations {
            enabled: false,
            approval_required: true,
            reason_required: true,
            message: None,
            min_age: 10,
        },
        contact: Contact {
            email: "no".to_string(),
        },
    })
}
