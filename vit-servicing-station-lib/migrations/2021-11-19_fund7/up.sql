ALTER TABLE funds ADD COLUMN next_registration_snapshot_time BIGINT NOT NULL DEFAULT 0;

ALTER TABLE challenges ADD COLUMN highlights VARCHAR;

ALTER TABLE community_advisors_reviews ADD COLUMN impact_alignment_rating_given INTEGER NOT NULL;
ALTER TABLE community_advisors_reviews ADD COLUMN impact_alignment_note VARCHAR NOT NULL;
ALTER TABLE community_advisors_reviews ADD COLUMN feasibility_rating_given INTEGER NOT NULL;
ALTER TABLE community_advisors_reviews ADD COLUMN feasibility_note VARCHAR NOT NULL;
ALTER TABLE community_advisors_reviews ADD COLUMN auditability_rating_given INTEGER NOT NULL;
ALTER TABLE community_advisors_reviews ADD COLUMN auditability_note VARCHAR NOT NULL;

UPDATE community_advisors_reviews SET impact_alignment_rating_given = rating_given;
UPDATE community_advisors_reviews SET feasibility_rating_given = rating_given;
UPDATE community_advisors_reviews SET auditability_rating_given = rating_given;
UPDATE community_advisors_reviews SET impact_alignment_note = note;
UPDATE community_advisors_reviews SET feasibility_note = note;
UPDATE community_advisors_reviews SET auditability_note = note;

ALTER TABLE community_advisors_reviews DROP tag;
ALTER TABLE community_advisors_reviews DROP note;
ALTER TABLE community_advisors_reviews DROP rating_given;