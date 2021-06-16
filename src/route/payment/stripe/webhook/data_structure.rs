use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum EventType {
    #[serde(rename = "customer.subscription.created")]
    CustomerSubscriptionCreated,
    #[serde(rename = "customer.subscription.deleted")]
    CustomerSubscriptionDeleted,
    #[serde(rename = "customer.subscription.trial_will_end")]
    CustomerSubscriptionTrialWillEnd,
    #[serde(rename = "customer.subscription.updated")]
    CustomerSubscriptionUpdated,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    NotStarted,
    Active,
    Completed,
    Released,
    Canceled,
    Trialing,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct SubscriptionPhase {
    pub end_date: i64,
    pub start_date: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScheduleObject {
    pub id: String,
    pub current_phase: Option<SubscriptionPhase>,
    pub customer: crate::database::stripe_profile::StripeID,
    pub status: SubscriptionStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubscriptionObject {
    pub id: String,
    pub customer: crate::database::stripe_profile::StripeID,
    pub status: SubscriptionStatus,
    pub current_period_start: i64,
    pub current_period_end: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "object", rename_all = "snake_case")]
pub enum EventObject {
    Schedule(ScheduleObject),
    Subscription(SubscriptionObject),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventData {
    pub object: EventObject,
}

#[derive(Deserialize, Debug)]
pub struct Event {
    #[serde(rename = "type")]
    pub event_type: Option<EventType>,
    pub data: EventData,
    pub livemode: bool,
}
