-- This file should undo anything in `up.sql`
ALTER TABLE funds DROP COLUMN insight_sharing_start;
ALTER TABLE funds DROP COLUMN proposal_submission_start;
ALTER TABLE funds DROP COLUMN refine_proposals_start;
ALTER TABLE funds DROP COLUMN finalize_proposals_start;
ALTER TABLE funds DROP COLUMN proposal_assessment_start;
ALTER TABLE funds DROP COLUMN assessment_qa_start;
ALTER TABLE funds DROP COLUMN snapshot_start;
ALTER TABLE funds DROP COLUMN voting_start;
ALTER TABLE funds DROP COLUMN voting_end;
ALTER TABLE funds DROP COLUMN tallying_end;
ALTER TABLE funds DROP COLUMN results_url;
ALTER TABLE funds DROP COLUMN survey_url;

ALTER TABLE challenges DROP COLUMN internal_id;
ALTER TABLE challenges DROP UNIQUE id;

DROP table goals IF EXISTS;