use crate::TransferWiseError;
use macros_create_app::make_app65;
use macros_make_model::make_model22;
use macros_make_scope::make_scope;
use my_schema::general::IProduct;
use my_state::MyState;
use serde::*;
use sqlx::Error;

make_model22!(
    QTransferWisePayment,
    ITransferWisePayment,
    OTransferWisePayment,
    transferwise_payment,
    products: sqlx::types::Json<Vec<IProduct>>,
    reference: String,
    amount: bigdecimal::BigDecimal,
    created_at: chrono::DateTime<chrono::Utc>
);

#[derive(
    utoipa::ToSchema, Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone, Default,
)]
pub struct TransferWisePaymentRequest {
    pub data: ITransferWisePayment,
    pub wallet_request: lib_wallet::WalletAuthId,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
struct IdPathParam {
    pub id: i32,
}

make_app65!(
    [
        products: sqlx::types::Json<Vec<IProduct>>,
        reference: String,
        amount: bigdecimal::BigDecimal,
        created_at: chrono::DateTime<chrono::Utc>
    ],
    wise_payment,
    "/transferwise/payment",
    "/transferwise/payment/{id}",
    "/payment",
    "/payment/{id}",
    OTransferWisePayment,
    QTransferWisePayment,
    transferwise_payment,
    [
        TransferWisePaymentRequest,
        |my_state: actix_web::web::Data<MyState>,
         json: actix_web::web::Json<TransferWisePaymentRequest>,
         wallet: lib_wallet::QWallet,
         http_request: actix_web::HttpRequest| async move { handle(my_state, json).await }
    ],
    TransferWiseError
);

make_scope!("transferwise", [post, wise_payment]);

async fn handle(
    s: actix_web::web::Data<MyState>,
    json: actix_web::web::Json<TransferWisePaymentRequest>,
) -> Result<QTransferWisePayment, TransferWiseError> {
    transferwise_payment::postgres_query::insert(&s.sqlx_pool, &json.data)
        .await
        .map_err(TransferWiseError::from_general)
}
