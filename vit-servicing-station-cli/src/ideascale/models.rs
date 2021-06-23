use serde::Serialize;

#[derive(Debug, Deserialize)]
pub struct Challenge {
    id: u64,
    #[serde(alias = "name")]
    title: String,
    description: String,
    #[serde(alias = "groupId")]
    fund_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct Fund {
    id: u64,
    name: String,
    #[serde(alias = "campaigns")]
    challenges: Vec<Challenge>,
}

#[derive(Debug, Deserialize)]
pub struct Proposal {
    #[serde(alias = "id")]
    proposal_id: i32,
    proposal_category: String,
    #[serde(alias = "title")]
    proposal_title: String,
    #[serde(alias = "text")]
    proposal_summary: String,
    #[serde(alias = "describe_your_solution_to_the_problem")]
    proposal_solution: String,
    #[serde(alias = "ada_payment_address__must_be_a_shelly_address__starting_with__addr__")]
    proposal_public_key: String,
    #[serde(alias = "requested_funds_in_ada")]
    proposal_funds: i64,
    #[serde(alias = "url")]
    proposal_url: String,
    #[serde(default)]
    proposal_files_url: String,
    #[serde(default)]
    proposal_impact_score: i64,
    #[serde(alias = "relevant_experience")]
    proposal_relevant_experience: String,

    #[serde(alias = "why_is_it_important_")]
    proposal_why: String,
}
