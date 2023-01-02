use macros_create_app::make_app33;
use macros_make_error::make_error;
use macros_make_scope::make_scope;

make_error!(MyError);

make_app33!(
    [
        data: sqlx::types::Json<serde_json::Value>,
        subscription_id: String,
        event_type: String,
        sent_at: DateTime<Utc>
    ],
    deposit,
    "/transferwise/deposit",
    "/transferwise/deposit/{id}",
    "/deposit",
    "/deposit/{id}",
    OTransferWiseDeposit,
    QTransferWiseDeposit,
    transferwise_deposit,
    [
        ITransferWiseDeposit,
        no_auth | my_state: actix_web::web::Data<MyState>,
        json: actix_web::web::Json<ITransferWiseDeposit>
            | async move {
                println!("{:?}", json);
                Ok::<QTransferWiseDeposit, MyError>(QTransferWiseDeposit::default())
            }
    ]
);

make_scope!("transferwise", [post, deposit]);

fn handle() {}
