CREATE TYPE expense_status AS ENUM ('pending', 'approved', 'rejected');

CREATE TABLE expenses (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category_id  UUID NOT NULL REFERENCES categories(id),
    amount       INTEGER NOT NULL,
    purpose      TEXT NOT NULL,
    occurred_at  DATE NOT NULL,
    note         TEXT,
    receipt_file VARCHAR(512),
    status       expense_status NOT NULL DEFAULT 'pending',
    created_at   TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_expenses_user_id ON expenses(user_id);
CREATE INDEX idx_expenses_status ON expenses(status);
