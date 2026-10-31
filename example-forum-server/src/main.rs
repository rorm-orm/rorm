#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    rorm::testing::postgres_tmp_dir(|db| async move {
        let () = ();
        Ok(())
    })
    .await
}
