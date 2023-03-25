use actix_web::{HttpRequest, HttpResponse, Responder};

#[tracing::instrument(
    name = "health check", // 未指定, 默认使用函数名称
    skip(_req), // 自动将函数参数添加直上下文, 明确忽略参数
)]
pub async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().finish()
}
