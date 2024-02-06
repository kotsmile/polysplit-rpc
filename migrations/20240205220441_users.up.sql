-- Add up migration script here
create table users (
    id uuid not null,
    email varchar not null,
    primary key (id)
);
