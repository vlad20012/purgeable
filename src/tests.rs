use crate::{NonPurgeableBox, PurgeableBox};

#[test]
fn box_impls_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<PurgeableBox<i32>>();
    assert_send::<NonPurgeableBox<i32>>();

    assert_sync::<NonPurgeableBox<i32>>();
}

#[test]
fn alloc_zst() {
    let _ = NonPurgeableBox::new(&());
}

#[test]
fn test_deref() {
    let l = NonPurgeableBox::new(&1i32);
    assert_eq!(*l, 1);
    let u = NonPurgeableBox::unlock(l);
    if let Ok(mut l) = u.lock() {
        assert_eq!(*l, 1);
        *l = 2;
        assert_eq!(*l, 2);
        let u = NonPurgeableBox::unlock(l);
        if let Ok(l) = u.lock() {
            assert_eq!(*l, 2);
        }
    }
}
