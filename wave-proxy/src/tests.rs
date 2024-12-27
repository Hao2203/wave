use super::*;

#[tokio::test]
async fn test() {
    use crate::socks5::*;

    println!("test");
    let s = Socks5::new("172.27.197.215:555".parse().unwrap());
    let mut proxy = Proxy::new(s).await.unwrap();

    while let Some(res) = proxy.next().await {
        let Incoming { target, io } = res.unwrap();
        drop(io);
        println!("{:?}", target);
    }
}
