use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing;
use tracing::Instrument;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}
#[tracing::instrument(
    name = "Adding a new subscriber", // 未指定, 默认使用函数名称
    skip(form, connection), // 自动将函数参数添加直上下文, 明确忽略参数
    fields( // fields 指令来丰富 span 的上下文. 
    subscriber_email = %form.email,
    subscriber_name= %form.name
    )
    )]
pub async fn subscribe(connection: web::Data<PgPool>, form: web::Form<FormData>) -> HttpResponse {
    /*
     使用 tracing::instrument 宏
        let request_id = Uuid::new_v4();

        // Spans, like logs, have an associated level
        // `info_span` creates a span at the info-level
        let request_span = tracing::info_span!(
            "Adding a new subscriber.",
            %request_id,
            subscriber_email = %form.email,
            subscriber_name= %form.name
        );

        // Using `enter` in an async function is a recipe for disaster!
        // Bear with me for now, but don't do this at home.
        // See the following section on `Instrumenting Futures`
        //
        // `_request_span_guard` is dropped at the end of `subscribe`
        // That's when we "exit" the span
        let _request_span_guard = request_span.enter();
    */

    // We do not call `.enter` on query_span!
    // `.instrument` takes care of it at the right moments
    // in the query future lifetime
    let query_span = tracing::info_span!("Saving new subscriber details in the database");

    let result = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    // We use `get_ref` to get an immutable reference to the `PgConnection`
    // wrapped by `web::Data`.
    .execute(connection.as_ref())
    .instrument(query_span)
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}
