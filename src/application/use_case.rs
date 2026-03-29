use async_trait::async_trait;

#[async_trait]
pub trait UseCase<Input, Output> {
    type Error: std::error::Error + Send + Sync;
    async fn execute(&self, input: Input) -> Result<Output, Self::Error>;
}
