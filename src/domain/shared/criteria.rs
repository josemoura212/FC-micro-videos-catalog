pub trait Criteria<Context>: Send + Sync {
    fn apply(&self, context: Context) -> Context;
}

pub trait ScopedRepository {
    fn ignore_soft_deleted(&mut self) -> &mut Self;
    fn clear_scopes(&mut self) -> &mut Self;
}
