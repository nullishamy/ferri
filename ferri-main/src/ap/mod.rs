pub type ObjectId = String;

pub enum ObjectType {
	Person,
}

pub struct Object {
	id: ObjectId,
	ty: ObjectType
}

pub struct Actor {
	obj: Object,
	
	inbox: Inbox,
	outbox: Outbox,
}

pub struct Inbox {}

pub struct Outbox {}

pub struct Message {}

pub struct Activity {
	
}
