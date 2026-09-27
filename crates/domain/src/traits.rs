pub trait Apply<Event, Output = Self> {
    fn apply(self, event: Event) -> Output;
}

pub trait Aggregate : Sized where Self: Apply<Self::Event> {
    type Command;
    type Event;
    type Error;
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, Self::Error>;
}