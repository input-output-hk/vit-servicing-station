DROP VIEW IF EXISTS "full_proposals_info";
CREATE VIEW full_proposals_info
AS
SELECT
    proposals.*,
    ifnull(reviews_count, 0) as reviews_count,
    proposal_simple_challenge.proposal_solution,
    proposal_community_choice_challenge.proposal_brief,
    proposal_community_choice_challenge.proposal_importance,
    proposal_community_choice_challenge.proposal_goal,
    proposal_community_choice_challenge.proposal_metrics,
    voteplans.chain_vote_start_time,
    voteplans.chain_vote_end_time,
    voteplans.chain_committee_end_time,
    voteplans.chain_voteplan_payload,
    voteplans.chain_vote_encryption_key,
    voteplans.fund_id,
    challenges.challenge_type
FROM
    proposals
        INNER JOIN voteplans ON proposals.chain_voteplan_id = voteplans.chain_voteplan_id
        INNER JOIN challenges on challenges.id = proposals.challenge_id
        LEFT JOIN proposal_simple_challenge
            on proposals.proposal_id = proposal_simple_challenge.proposal_id
            and (challenges.challenge_type = 'simple' or challenges.challenge_type = 'native')
        LEFT JOIN proposal_community_choice_challenge
            on proposals.proposal_id = proposal_community_choice_challenge.proposal_id
            and challenges.challenge_type = 'community-choice'
        LEFT JOIN (SELECT proposal_id as review_proposal_id, COUNT (DISTINCT assessor) as reviews_count FROM community_advisors_reviews GROUP BY proposal_id)
            on proposals.proposal_id = review_proposal_id;