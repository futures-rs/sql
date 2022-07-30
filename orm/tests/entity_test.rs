use rdbc_attributes::*;
use rdbc_orm::Serialize;

#[allow(dead_code)]
#[derive(ORM, Default)]
struct TestTable<Id>
where
    Id: Serialize,
{
    id: Id,
    data: Data,
}

#[allow(dead_code)]
#[derive(ORM, Default)]
struct Data {
    #[column(name = "hello")]
    id: i32,
}

#[test]
fn test_serializer() {
    _ = pretty_env_logger::try_init();

    let table = TestTable::<i32>::default();
}
