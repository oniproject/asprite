pub trait Model {
	type Model;
	type Message;
	fn model() -> Self::Model;
	fn update(&self, event: Self::Msg);
}
