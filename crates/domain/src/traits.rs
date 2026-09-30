pub trait Apply<Event, Output = Self> {
    fn apply(self, event: Event) -> Output;
}

pub trait Aggregate: Sized + Apply<Self::Event> {
    type Id: Copy + Eq + std::hash::Hash + Send + Sync + 'static;
    type Command;
    type Event;
    type Error;
    fn id(&self) -> Self::Id;
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
}
