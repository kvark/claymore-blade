create table if not exists hunts (
  user_id text primary key,
  save jsonb not null,
  updated_at timestamptz not null default now()
);
