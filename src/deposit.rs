use crate::TransferWiseError;
use macros_create_app::make_app65;
use macros_make_model::make_model22;
use macros_make_scope::make_scope;
use my_state::MyState;
use serde::*;

make_model22!(
    Q,
    I,
    O,
    transferwise_deposit,
    data: sqlx::types::Json<serde_json::Value>,
    subscription_id: String,
    event_type: String,
    sent_at: chrono::DateTime<chrono::Utc>
);

#[derive(
    utoipa::ToSchema, Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone, Default,
)]
pub struct TransferWiseDepositRequest {
    pub data: I,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
struct IdPathParam {
    pub id: i32,
}

make_app65!(
    [
        data: sqlx::types::Json<serde_json::Value>,
        subscription_id: String,
        event_type: String,
        sent_at: chrono::DateTime<chrono::Utc>
    ],
    route,
    "/transferwise/deposit",
    "/transferwise/deposit/{id}",
    "/deposit",
    "/deposit/{id}",
    O,
    Q,
    transferwise_deposit,
    [
        I,
        no_auth | my_state: actix_web::web::Data<MyState>,
        json: actix_web::web::Json<I>
            | async move {
                println!("{:?}", json);
                Ok::<Q, TransferWiseError>(Q::default())
            }
    ],
    TransferWiseError
);

make_scope!("transferwise", [post, route]);

fn handle(
    my_state: actix_web::web::Data<MyState>,
    json: actix_web::web::Json<I>,
) {
}
