CREATE TABLE logs (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    message TEXT NOT NULL
);
