use std::collections::HashMap;
use std::marker::PhantomData;

pub enum DBErrors {
    NotImplemented,
}

pub struct DBConnectorConfig {
    pub authDetails: HashMap<String, String>,
    pub address: String, // the format of this dependends on the implementation
    pub port: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DBQueryOps {
    Equal,
    NotEqual,
    BiggerThan,
    SmallerThan,
    BiggerOrEqual,
    SmallerOrEqual,
}

/// Typed query handle for a specific implementation (`DBServer`, `Db`, `DBTable`, etc.).
///
/// `kind` is the query semantic (from the generated method name). `filter` is the
/// user-provided operand. Interpretation stays in each implementation.
#[derive(Debug, PartialEq, Eq)]
pub struct DBQuery<I> {
    pub(crate) kind: &'static str,
    pub(crate) op: DBQueryOps,
    pub(crate) filter: String,
    _impl: PhantomData<I>,
}

impl<I> DBQuery<I> {
    pub(crate) fn new(kind: &'static str, op: DBQueryOps, filter: &str) -> Self {
        Self {
            kind,
            op,
            filter: filter.to_string(),
            _impl: PhantomData,
        }
    }
}

pub trait DBRow: Sized {}

pub trait DBTable<R: DBRow> {
    const TABLE_NAME: &'static str;
}

pub trait Db<B: DBRow, T: DBTable<B>> {}

pub trait DBServer<A: DBTable<R>, R: DBRow, T: Db<R, A>> {
    fn getDBs(&self) -> Result<T, DBErrors>;
    fn getDB(&self, name: String) -> Result<T, DBErrors>;
}

pub trait DBConnector<R: DBRow, A: DBTable<R>, D: Db<R, A>, T: DBServer<A, R, D>> {
    fn connect(cfg: DBConnectorConfig) -> Result<T, DBErrors>;
}

/// Generates `query_<name>(op, filter)` helpers on an implementation type.
///
/// Each generated method takes a [`DBQueryOps`] and a filter string, and returns a
/// [`DBQuery`] tagged with that semantic `kind`.
///
/// # Example
///
/// ```ignore
/// impl_db_query! {
///     on: PostgresServer,
///     db_name,
/// }
///
/// let q = PostgresServer::query_db_name(DBQueryOps::Equal, "vacation_app");
/// ```
#[macro_export]
macro_rules! impl_db_query {
    (
        on: $impl_ty:ty,
        $( $method:ident ),+ $(,)?
    ) => {
        impl $impl_ty {
            $crate::impl_db_query! { @each $impl_ty, $( $method ),+ }
        }
    };

    (@each $impl_ty:ty, $method:ident $(, $rest:ident)* $(,)? ) => {
        paste::paste! {
            pub fn [<query_ $method>](
                op: $crate::db::DBQueryOps,
                filter: &str,
            ) -> $crate::db::DBQuery<$impl_ty> {
                $crate::db::DBQuery::<$impl_ty>::new(stringify!($method), op, filter)
            }
        }
        $crate::impl_db_query! { @each $impl_ty, $( $rest ),* }
    };

    (@each $impl_ty:ty,) => {};
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct PostgresServer;

    impl_db_query! {
        on: PostgresServer,
        db_name,
    }

    pub struct PostgresDatabase;

    impl_db_query! {
        on: PostgresDatabase,
        table_name,
    }

    #[test]
    fn query_helpers_store_kind_op_and_filter() {
        let q = PostgresServer::query_db_name(DBQueryOps::Equal, "vacation_app");
        assert_eq!(q.kind, "db_name");
        assert_eq!(q.op, DBQueryOps::Equal);
        assert_eq!(q.filter, "vacation_app");

        let table = PostgresDatabase::query_table_name(DBQueryOps::NotEqual, "sessions");
        assert_eq!(table.kind, "table_name");
        assert_eq!(table.op, DBQueryOps::NotEqual);
        assert_eq!(table.filter, "sessions");
    }
}
