CREATE TABLE greetings (
    id         BIGSERIAL PRIMARY KEY,
    name       TEXT NOT NULL,
    greeted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
