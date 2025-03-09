//! This module is for defining kv store with index.

#[macro_export]
macro_rules! define_store {
    (

    ) => {

        impl Store
    };
}

define_store! {
    table: task,
    index: [
        (done, &path, start, due) -> id,
    ],
}
