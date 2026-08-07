use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "u8", db_type = "Integer")]
pub enum UserStatus {
    #[sea_orm(num_value = 0)]
    Disabled,
    #[sea_orm(num_value = 1)]
    Enabled,
    #[sea_orm(num_value = 2)]
    WaitingEmailVerification,
}


#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "t_user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    #[sea_orm(column_name = "first_name")]
    pub first_name: String,
    #[sea_orm(column_name = "last_name")]
    pub last_name: String,
    #[sea_orm(column_name = "email")]
    pub email: String,
    #[sea_orm(column_name = "status")]
    pub status: UserStatus,
}


impl ActiveModelBehavior for ActiveModel {}