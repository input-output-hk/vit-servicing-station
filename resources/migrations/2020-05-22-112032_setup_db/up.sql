DROP TABLE IF EXISTS "__diesel_schema_migrations";
CREATE TABLE IF NOT EXISTS "__diesel_schema_migrations" (
	"version"	VARCHAR(50) NOT NULL,
	"run_on"	TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY("version","run_on")
);
DROP TABLE IF EXISTS "funds";
CREATE TABLE IF NOT EXISTS "funds" (
	"id"	INTEGER NOT NULL,
	"fund_name"	VARCHAR NOT NULL,
	"fund_goal"	VARCHAR NOT NULL,
	"registration_snapshot_time"	BIGINT,
	"next_registration_snapshot_time"	BIGINT ,
	"voting_power_threshold"	BIGINT,
	"fund_start_time"	BIGINT,
	"fund_end_time"	BIGINT,
	"next_fund_start_time"	BIGINT,
	"insight_sharing_start"	BIGINT,
	"proposal_submission_start"	BIGINT,
	"refine_proposals_start"	BIGINT,
	"finalize_proposals_start"	BIGINT,
	"proposal_assessment_start"	BIGINT,
	"assessment_qa_start"	BIGINT,
	"snapshot_start"	BIGINT,
	"voting_start"	BIGINT,
	"voting_end"	BIGINT,
	"tallying_end"	BIGINT,
	"results_url"	VARCHAR,
	"survey_url"	VARCHAR,
	PRIMARY KEY("id")
);
DROP TABLE IF EXISTS "proposals";
CREATE TABLE IF NOT EXISTS "proposals" (
	"id" INTEGER NOT NULL,
	"proposal_id"	VARCHAR NOT NULL,
	"proposal_category"	VARCHAR NOT NULL,
	"proposal_title"	VARCHAR NOT NULL,
	"proposal_summary"	VARCHAR NOT NULL,
	"proposal_public_key"	VARCHAR NOT NULL,
	"proposal_funds"	BIGINT NOT NULL,
	"proposal_url"	VARCHAR NOT NULL,
	"proposal_files_url"	VARCHAR NOT NULL,
	"proposal_impact_score"	BIGINT NOT NULL,
	"proposer_name"	VARCHAR NOT NULL,
	"proposer_contact"	VARCHAR NOT NULL,
	"proposer_url"	VARCHAR NOT NULL,
	"proposer_relevant_experience"	VARCHAR NOT NULL,
	"chain_proposal_id"	BLOB NOT NULL,
	"chain_proposal_index"	BIGINT NOT NULL,
	"chain_vote_options"	VARCHAR NOT NULL,
	"chain_voteplan_id"	VARCHAR NOT NULL,
	"challenge_id"	INTEGER NOT NULL,
	PRIMARY KEY("id","proposal_id")
);
DROP TABLE IF EXISTS "proposal_simple_challenge";
CREATE TABLE IF NOT EXISTS "proposal_simple_challenge" (
	"proposal_id"	VARCHAR NOT NULL,
	"proposal_solution"	VARCHAR,
	PRIMARY KEY("proposal_id")
);
DROP TABLE IF EXISTS "proposal_community_choice_challenge";
CREATE TABLE IF NOT EXISTS "proposal_community_choice_challenge" (
	"proposal_id"	VARCHAR NOT NULL,
	"proposal_brief"	VARCHAR,
	"proposal_importance"	VARCHAR,
	"proposal_goal"	VARCHAR,
	"proposal_metrics"	VARCHAR,
	PRIMARY KEY("proposal_id")
);
DROP TABLE IF EXISTS "voteplans";
CREATE TABLE IF NOT EXISTS "voteplans" (
	"id"	INTEGER NOT NULL,
	"chain_voteplan_id"	VARCHAR NOT NULL UNIQUE,
	"chain_vote_start_time"	BIGINT NOT NULL,
	"chain_vote_end_time"	BIGINT NOT NULL,
	"chain_committee_end_time"	BIGINT NOT NULL,
	"chain_voteplan_payload"	VARCHAR NOT NULL,
	"chain_vote_encryption_key"	VARCHAR NOT NULL,
	"fund_id"	INTEGER NOT NULL,
	PRIMARY KEY("chain_voteplan_id")
);
DROP TABLE IF EXISTS "api_tokens";
CREATE TABLE IF NOT EXISTS "api_tokens" (
	"token"	BLOB NOT NULL UNIQUE,
	"creation_time"	BIGINT NOT NULL,
	"expire_time"	BIGINT NOT NULL,
	PRIMARY KEY("token")
);
DROP TABLE IF EXISTS "challenges";
CREATE TABLE IF NOT EXISTS "challenges" (
	"internal_id"	INTEGER,
	"id"	INTEGER NOT NULL,
	"challenge_type"	VARCHAR NOT NULL,
	"title"	VARCHAR NOT NULL,
	"description"	VARCHAR NOT NULL,
	"rewards_total"	BIGINT NOT NULL,
	"proposers_rewards"	BIGINT NOT NULL,
	"fund_id"	INTEGER NOT NULL,
	"challenge_url"	VARCHAR NOT NULL,
	"highlights"	VARCHAR,
	PRIMARY KEY("id","challenge_url")
);
DROP TABLE IF EXISTS "community_advisors_reviews";
CREATE TABLE IF NOT EXISTS "community_advisors_reviews" (
	"id"	INTEGER NOT NULL,
	"proposal_id"	INTEGER NOT NULL,
	"assessor"	VARCHAR NOT NULL,
	"impact_alignment_rating_given"	INTEGER,
	"impact_alignment_note"	VARCHAR,
	"feasibility_rating_given"	INTEGER,
	"feasibility_note"	VARCHAR,
	"auditability_rating_given"	INTEGER,
	"auditability_note"	VARCHAR,
	"ranking"	INTEGER,
	"rating_given" INTEGER, 
	"tag" VARCHAR,
	"note" VARCHAR,
	PRIMARY KEY("id","proposal_id")
);
DROP TABLE IF EXISTS "goals";
CREATE TABLE IF NOT EXISTS "goals" (
	"id"	INTEGER NOT NULL,
	"goal_name"	VARCHAR NOT NULL,
	"fund_id"	INTEGER NOT NULL,
	PRIMARY KEY("id","fund_id")
);
DROP TABLE IF EXISTS "votes";
CREATE TABLE "votes" (
	"fragment_id"	TEXT,
	"caster"	TEXT,
	"proposal"	INTEGER,
	"voteplan_id"	TEXT,
	"time"	REAL,
	"choice"	TEXT,
	"raw_fragment"	TEXT,
	PRIMARY KEY("fragment_id")
)