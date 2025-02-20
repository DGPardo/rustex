mod test_buy_get_delete;

#[tokio::main]
pub async fn main() {
    test_buy_get_delete::test_buy_get_delete().await
}
