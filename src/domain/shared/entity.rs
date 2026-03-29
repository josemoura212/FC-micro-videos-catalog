use super::notification::Notification;
use super::value_object::UuidVo;

pub trait Entity: Send + Sync {
    fn entity_id(&self) -> &UuidVo;
    fn notification(&self) -> &Notification;
    fn notification_mut(&mut self) -> &mut Notification;
}

pub trait AggregateRoot: Entity {}
