/*
What should we handle here:
- Inbound/Outbound queues
- Accepting events from the API to process
  - Which entails the logic for all of that
- Remote actioning (webfinger, httpwrapper if possible)
 */

mod request_queue;
pub use request_queue::*;

pub mod inbox;
pub mod outbox;
pub mod http;

