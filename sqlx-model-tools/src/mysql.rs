use sqlx::{MySql, Pool, Row};

use crate::common::{ConfigParseError, DataField, TableParseData};

#[cfg(feature = "sqlx-mysql")]
pub(crate) struct MySqlParse(Pool<MySql>);
#[cfg(feature = "sqlx-mysql")]
#[async_trait::async_trait]
impl TableParseData for MySqlParse {
    type OUT = MySqlParse;
    async fn new(uri: &str) -> Result<Self::OUT, ConfigParseError> {
        let db = sqlx::pool::PoolOptions::<sqlx::MySql>::new()
            .connect(uri)
            .await?;
        Ok(Self(db))
    }
    async fn list_tables(&self) -> Result<Vec<String>, ConfigParseError> {
        let mut tables = vec![];
        let res = sqlx::query("show tables");
        let rows = res.fetch_all(&self.0).await?;
        for row in rows {
            let tablename = row.get::<&str, _>(0).to_owned();
            tables.push(tablename);
        }
        Ok(tables)
    }
    async fn parse_table_column(
        &self,
        table_name: &str,
    ) -> Result<Vec<DataField>, ConfigParseError> {
        let mut columns = vec![];
        let sql = format!(" show full columns from {}", table_name);
        let res = sqlx::query(sql.as_str());
        let rows = res.fetch_all(&self.0).await?;
        for row in rows {
            let name = row.get::<&str, _>("Field").to_string();
            let ty = row.get::<&str, _>("Type").to_owned();
            let mut is_null = false;
            let key = row.get::<&str, _>("Null");
            if key.contains("YES") {
                is_null = true;
            }
            let mut is_pk = false;
            let key = row.get::<&str, _>("Key");
            if key.contains("PRI") {
                is_pk = true;
            }
            let def = if is_null || is_pk {
                row.try_get::<Option<&str>, _>("Default")
                    .unwrap_or_default()
                    .map(|e| e.to_owned())
            } else {
                row.try_get::<&str, _>("Default").map(|e| e.to_owned()).ok()
            };

            let comment = row.get::<&str, _>("Comment");

            let column = DataField {
                field_name: name,
                type_name: ty,
                is_null,
                default: def,
                comment: comment.to_owned(),
                is_pk,
            };
            columns.push(column.clone());
        }
        Ok(columns)
    }
}
