use rust_mai::dev_read::*;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let hndl = start_key_listen(tx).await;
    while let Some(event) = rx.recv().await {
        println!("{event:#?}");
    }
    hndl.await.unwrap();
}
