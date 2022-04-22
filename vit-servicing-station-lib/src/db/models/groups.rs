use crate::db::schema::groups;
use diesel::{ExpressionMethods, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Queryable)]
pub struct Group {
    #[serde(alias = "groupId")]
    pub group_id: String,
    #[serde(alias = "tokenId")]
    pub token_identifier: String,
}

// This warning is disabled here. Values is only referenced as a type here. It should be ok not to
// split the types definitions.
#[allow(clippy::type_complexity)]
impl Insertable<groups::table> for Group {
    type Values = (
        diesel::dsl::Eq<groups::group_id, String>,
        diesel::dsl::Eq<groups::token_identifier, String>,
    );

    fn values(self) -> Self::Values {
        (
            groups::group_id.eq(self.group_id),
            groups::token_identifier.eq(self.token_identifier),
        )
    }
}
