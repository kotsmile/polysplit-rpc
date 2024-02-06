-- Add up migration script here
create table users (
    id uuid not null unique,

    email varchar not null unique,

    primary key (id)
);
