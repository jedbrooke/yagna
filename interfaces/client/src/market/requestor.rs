//! Requestor part of Market API
use std::sync::Arc;

use crate::{web::WebClient, Result};
use ya_model::market::{Agreement, AgreementProposal, Demand, Event, Proposal};

/// Bindings for Requestor part of the Market API.
#[derive(Clone)]
pub struct RequestorApi {
    client: Arc<WebClient>,
}

impl RequestorApi {
    pub fn new(client: &Arc<WebClient>) -> Self {
        Self {
            client: client.clone(),
        }
    }

    /// Publishes Requestor capabilities via Demand.
    ///
    /// Demand object can be considered an "open" or public Demand, as it is not directed
    /// at a specific Provider, but rather is sent to the market so that the matching
    /// mechanism implementation can associate relevant Offers.
    ///
    /// **Note**: it is an "atomic" operation, ie. as soon as Subscription is placed,
    /// the Demand is published on the market.
    pub async fn subscribe_demand(&self, demand: &Demand) -> Result<String> {
        self.client.post("demands/").send_json(&demand).json().await
    }

    /// Stop subscription by invalidating a previously published Demand.
    pub async fn unsubscribe_demand(&self, subscription_id: &str) -> Result<String> {
        let url = url_format!("demands/{subscription_id}/", subscription_id);
        self.client.delete(&url).send().json().await
    }

    /// Get events which have arrived from the market in response to the Demand
    /// published by the Requestor via  [`subscribe`](#method.subscribe).
    /// Returns collection of at most `max_events` `RequestorEvents` or times out.
    #[rustfmt::skip]
    pub async fn collect_offers(
        &self,
        subscription_id: &str,
        timeout: Option<i32>,
        #[allow(non_snake_case)]
        maxEvents: Option<i32>, // TODO: max_events
    ) -> Result<Vec<Event>> {
        let url = url_format!(
            "demands/{subscription_id}/events/",
            subscription_id,
            #[query] timeout,
            #[query] maxEvents
        );
        self.client.get(&url).send().json().await
    }

    /// Responds with a bespoke Demand to received Offer.
    pub async fn create_proposal_demand(
        &self,
        proposal: &Proposal,
        subscription_id: &str,
        proposal_id: &str,
    ) -> Result<String> {
        let url = url_format!(
            "demands/{subscription_id}/proposals/{proposal_id}/demand/",
            subscription_id,
            proposal_id
        );
        self.client.post(&url).send_json(&proposal).json().await
    }

    /// Fetches Proposal (Offer) with given id.
    pub async fn get_proposal_offer(
        &self,
        subscription_id: &str,
        proposal_id: &str,
    ) -> Result<Proposal> {
        let url = url_format!(
            "demands/{subscription_id}/proposals/{proposal_id}/",
            subscription_id,
            proposal_id
        );
        self.client.get(&url).send().json().await
    }

    /// Rejects a bespoke Demand.
    pub async fn reject_proposal_offer(
        &self,
        subscription_id: &str,
        proposal_id: &str,
    ) -> Result<String> {
        let url = url_format!(
            "demands/{subscription_id}/proposals/{proposal_id}/",
            subscription_id,
            proposal_id
        );
        self.client.delete(&url).send().json().await
    }

    /// Creates Agreement from selected Proposal.
    ///
    /// Initiates the Agreement handshake phase.
    ///
    /// Formulates an Agreement artifact from the Proposal indicated by the
    /// received Proposal Id.
    ///
    /// The Approval Expiry Date is added to Agreement artifact and implies
    /// the effective timeout on the whole Agreement Confirmation sequence.
    ///
    /// A successful call to `createAgreement` shall immediately be followed
    /// by a `confirmAgreement` and `waitForApproval` call in order to listen
    /// for responses from the Provider.
    ///
    /// **Note**: Moves given Proposal to `Approved` state.
    pub async fn create_agreement(&self, agreement: &AgreementProposal) -> Result<String> {
        self.client
            .post("agreements/")
            .send_json(&agreement)
            .json()
            .await
    }

    /// Fetches agreement with given agreement id.
    pub async fn get_agreement(&self, agreement_id: &str) -> Result<Agreement> {
        let url = url_format!("agreements/{agreement_id}/", agreement_id);
        self.client.get(&url).send().json().await
    }

    /// Sends Agreement draft to the Provider.
    /// Signs Agreement self-created via `createAgreement` and sends it to the Provider.
    pub async fn confirm_agreement(&self, agreement_id: &str) -> Result<String> {
        let url = url_format!("agreements/{agreement_id}/confirm/", agreement_id);
        self.client.post(&url).send().json().await
    }

    /// Waits for Agreement approval by the Provider.
    /// This is a blocking operation. The call may be aborted by Requestor caller
    /// code. After the call is aborted, another `waitForApproval` call can be
    /// raised on the same Agreement Id.
    ///
    /// It returns one of the following options:
    ///
    /// * `Ok` - Indicates that the Agreement has been approved by the Provider.
    /// - The Provider is now ready to accept a request to start an Activity
    /// as described in the negotiated agreement.
    /// - The Requestor’s corresponding `waitForApproval` call returns Ok after
    /// this on the Provider side.
    ///
    /// * `Rejected` - Indicates that the Provider has called `rejectAgreement`,
    /// which effectively stops the Agreement handshake. The Requestor may attempt
    /// to return to the Negotiation phase by sending a new Proposal.
    ///
    /// * `Cancelled` - Indicates that the Requestor himself has called
    /// `cancelAgreement`, which effectively stops the Agreement handshake.
    pub async fn wait_for_approval(&self, agreement_id: &str) -> Result<String> {
        let url = url_format!("agreements/{agreement_id}/wait/", agreement_id);
        self.client.post(&url).send().json().await
    }

    /// Cancels agreement.
    /// Causes the awaiting `waitForApproval` call to return with `Cancelled` response.
    pub async fn cancel_agreement(&self, agreement_id: &str) -> Result<()> {
        let url = url_format!("agreements/{agreement_id}/", agreement_id);
        self.client.delete(&url).send().json().await
    }

    /// Terminates approved Agreement.
    pub async fn terminate_agreement(&self, agreement_id: &str) -> Result<String> {
        let url = url_format!("agreements/{agreement_id}/terminate/", agreement_id);
        self.client.post(&url).send().json().await
    }
}
