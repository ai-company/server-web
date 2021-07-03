use crate::{
    database::{stripe_profile, SqlPool},
    route::{EndpointProcessingError, EndpointProcessingResult},
};

use super::{EventObject, ScheduleObject, SubscriptionObject, SubscriptionStatus};

// On subscription change, either by ScheduleObject or SubscriptionObject, apply the change to the database
pub fn subscription_changed(input: EventObject, pool: &SqlPool) -> EndpointProcessingResult<()> {
    use EventObject::*;
    match input {
        Schedule(sche) => subscription_changed_sche(sche, pool),
        Subscription(sub) => subscription_changed_sub(sub, pool),
    }
}

fn subscription_changed_sche(
    schedule: ScheduleObject,
    pool: &SqlPool,
) -> EndpointProcessingResult<()> {
    use SubscriptionStatus::*;
    match schedule.status {
        Active | Trialing if schedule.current_phase.is_some() => {
            let phase = schedule.current_phase.unwrap();
            stripe_profile::set_subscription(
                &schedule.customer,
                phase.start_date,
                phase.end_date,
                pool,
            )?;
            Ok(())
        }
        Active => Err(EndpointProcessingError::RequestNonsensical),
        Canceled | Completed | Released | Trialing => Ok(stripe_profile::remove_subscription(
            &schedule.customer,
            pool,
        )?),
        NotStarted => {
            // Then we clear the current subscription if it exists, but don't care if it doesn't
            let _ = stripe_profile::remove_subscription(&schedule.customer, pool);
            Ok(())
        }
    }
}

fn subscription_changed_sub(
    subscription: SubscriptionObject,
    pool: &SqlPool,
) -> EndpointProcessingResult<()> {
    let SubscriptionObject {
        customer,
        current_period_start: start,
        current_period_end: end,
        ..
    } = subscription;
    use SubscriptionStatus::*;
    Ok(match subscription.status {
        Active | Trialing => stripe_profile::set_subscription(&customer, start, end, pool),
        Canceled | Completed | Released => stripe_profile::remove_subscription(&customer, pool),
        NotStarted => {
            // Then we clear the current subscription if it exists, but don't care if it doesn't
            let _ = stripe_profile::remove_subscription(&customer, pool);
            Ok(())
        }
    }?)
}
