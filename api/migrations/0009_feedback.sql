CREATE TABLE feedback (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    kind text NOT NULL CHECK (kind IN ('bug', 'idea', 'other')),
    body text NOT NULL,
    page text NOT NULL,
    site_version text NOT NULL,
    user_agent text,
    element jsonb,
    issue_number bigint,
    issue_url text,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX feedback_user_id_created_at_idx ON feedback (user_id, created_at);

CREATE TABLE feedback_images (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    feedback_id bigint NOT NULL UNIQUE REFERENCES feedback (id) ON DELETE CASCADE,
    bytes bytea NOT NULL,
    mime text NOT NULL CHECK (mime IN ('image/webp', 'image/png')),
    width integer NOT NULL CHECK (width BETWEEN 1 AND 4096),
    height integer NOT NULL CHECK (height BETWEEN 1 AND 4096),
    created_at timestamptz NOT NULL DEFAULT now()
);
