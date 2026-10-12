use crate::infrai_client::{InfraiClient, InfraiError};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatterStage {
    Intake,
    SignedDocumentDelivery,
    DeadlineFollowUp,
}

impl fmt::Display for MatterStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Intake => write!(f, "matter-intake"),
            Self::SignedDocumentDelivery => write!(f, "signed-document-delivery"),
            Self::DeadlineFollowUp => write!(f, "deadline-follow-up"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentFailure {
    IntakeRejected,
    DeliveryUnconfirmed,
    DeadlineMissed,
}

impl AgentFailure {
    pub fn stage(&self) -> MatterStage {
        match self {
            Self::IntakeRejected => MatterStage::Intake,
            Self::DeliveryUnconfirmed => MatterStage::SignedDocumentDelivery,
            Self::DeadlineMissed => MatterStage::DeadlineFollowUp,
        }
    }
}

impl fmt::Display for AgentFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IntakeRejected => write!(f, "required intake facts were not accepted"),
            Self::DeliveryUnconfirmed => write!(f, "signed document delivery was not confirmed"),
            Self::DeadlineMissed => write!(f, "deadline follow-up was not completed"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatterDisposition {
    HaltIntake,
    QueueDeliveryFollowUp,
    EscalateDeadline,
}

pub fn disposition_for(failure: &AgentFailure) -> MatterDisposition {
    match failure {
        AgentFailure::IntakeRejected => MatterDisposition::HaltIntake,
        AgentFailure::DeliveryUnconfirmed => MatterDisposition::QueueDeliveryFollowUp,
        AgentFailure::DeadlineMissed => MatterDisposition::EscalateDeadline,
    }
}

#[derive(Debug)]
pub enum LoopError {
    Agent(AgentFailure),
    Tracking(InfraiError),
}

pub async fn track_failure(
    client: &InfraiClient,
    matter_id: &str,
    failure: AgentFailure,
) -> Result<MatterDisposition, LoopError> {
    let stage = failure.stage();
    let exception = format!("matter={matter_id}; stage={stage}; failure={failure}");
    let event_key = format!("matter-{matter_id}-{stage}");
    client
        .errors_capture(&exception, &event_key)
        .await
        .map_err(LoopError::Tracking)?;
    Ok(disposition_for(&failure))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unconfirmed_signed_delivery_queues_follow_up_instead_of_advancing() {
        let decision = disposition_for(&AgentFailure::DeliveryUnconfirmed);

        assert_eq!(decision, MatterDisposition::QueueDeliveryFollowUp);
        assert_ne!(decision, MatterDisposition::EscalateDeadline);
    }
}
