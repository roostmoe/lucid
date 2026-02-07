use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    pg::Pg,
    query_builder::QueryId,
    serialize::{self, IsNull, Output, ToSql},
    AsExpression, FromSqlRow,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    io::Write,
};

use crate::schema::sql_types::DbIdentityType;

macro_rules! sql_conversion {
    (
        $sql_t:ident => $model_t:ident,
        $($to_matcher:tt => $to_result:tt),*,
    ) => {
        impl ToSql<$sql_t, Pg> for $model_t {
            fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
                match *self {
                    $($model_t::$to_matcher => out.write_all($to_result)?),*
                };

                Ok(IsNull::No)
            }
        }

        impl FromSql<$sql_t, Pg> for $model_t {
            fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
                match bytes.as_bytes() {
                    $($to_result => Ok($model_t::$to_matcher)),*,
                    x => Err(format!("Unrecognized {} variant {:?}", stringify!($sql_t), x).into()),
                }
            }
        }

        impl QueryId for $sql_t {
            type QueryId = $sql_t;
            const HAS_STATIC_QUERY_ID: bool = true;
        }
    };
}

#[derive(Debug, PartialEq, Clone, FromSqlRow, AsExpression, Serialize, Deserialize, JsonSchema)]
#[diesel(sql_type = DbIdentityType)]
#[serde(rename_all = "lowercase")]
pub enum IdentityType {
    User,
}

sql_conversion! {
    DbIdentityType => IdentityType,
    User => b"user",
}

impl Display for IdentityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityType::User => write!(f, "user"),
        }
    }
}
