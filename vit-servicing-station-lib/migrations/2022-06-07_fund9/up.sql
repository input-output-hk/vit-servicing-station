ALTER TABLE funds ADD COLUMN     insight_sharing_start BIGINT NOT NULL DEFAULT 0;
ALTER TABLE funds ADD COLUMN         proposal_submission_start BIGINT NOT NULL DEFAULT 0;
ALTER TABLE funds ADD COLUMN         refine_proposals_start BIGINT NOT NULL DEFAULT 0;
ALTER TABLE funds ADD COLUMN         finalize_proposals_start BIGINT NOT NULL DEFAULT 0;
ALTER TABLE funds ADD COLUMN         proposal_assessment_start BIGINT NOT NULL DEFAULT 0;
ALTER TABLE funds ADD COLUMN         assessment_qa_start BIGINT NOT NULL DEFAULT 0;
ALTER TABLE funds ADD COLUMN         snapshot_start BIGINT NOT NULL DEFAULT 0;
ALTER TABLE funds ADD COLUMN         voting_start BIGINT NOT NULL DEFAULT 0;
ALTER TABLE funds ADD COLUMN         voting_end BIGINT NOT NULL DEFAULT 0;
ALTER TABLE funds ADD COLUMN         tallying_end BIGINT NOT NULL DEFAULT 0;
ALTER TABLE funds ADD COLUMN         results_url VARCHAR NOT NULL DEFAULT "UNDEFINED";
ALTER TABLE funds ADD COLUMN         survey_url VARCHAR NOT NULL DEFAULT "UNDEFINED";

ALTER TABLE challenges ADD COLUMN internal_id INTEGER NOT NULL primary key autoincrement;
ALTER TABLE challenges ADD UNIQUE id;

create table goals
(
    id INTEGER NOT NULL
        primary key autoincrement,
    goal_name VARCHAR NOT NULL,
    fund_id INTEGER NOT NULL,
    FOREIGN KEY(fund_id) REFERENCES funds(id)
);


