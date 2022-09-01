-- This file should undo anything in `up.sql`
ALTER TABLE funds DROP COLUMN next_registration_snapshot_time;

ALTER TABLE challenges DROP COLUMN highlights;

ALTER TABLE community_advisors_reviews ADD COLUMN tag VARCHAR NOT NULL;
ALTER TABLE community_advisors_reviews ADD COLUMN note VARCHAR NOT NULL;
ALTER TABLE community_advisors_reviews ADD COLUMN rating_given INTEGER NOT NULL;

UPDATE community_advisors_reviews SET rating_given = impact_alignment_rating_given;
UPDATE community_advisors_reviews SET note = impact_alignment_note;

ALTER TABLE community_advisors_reviews DROP COLUMN impact_alignment_rating_given;
ALTER TABLE community_advisors_reviews DROP COLUMN impact_alignment_note;
ALTER TABLE community_advisors_reviews DROP COLUMN feasibility_rating_given;
ALTER TABLE community_advisors_reviews DROP COLUMN feasibility_note;
ALTER TABLE community_advisors_reviews DROP COLUMN auditability_rating_given;
ALTER TABLE community_advisors_reviews DROP COLUMN auditability_note;
