use bytes::{BufMut, Bytes, BytesMut};

#[test]
fn test() {
    let data = Bytes::from_static(b"Hello World");
    let mut buf = BytesMut::new();
    buf.put(data.clone());

    let ptr1 = data.as_ref().as_ptr();
    let ptr2 = buf.as_ref().as_ptr();
    let ptr3 = data.clone().as_ref().as_ptr();

    assert_eq!(ptr1, ptr3);
    assert_eq!(ptr1, ptr2);
}
