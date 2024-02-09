-- Add up migration script here
create table groups (
    id uuid not null unique,

    name varchar not null,
    owner_id uuid not null,
    api_key text not null default '',

    primary key (id),
    constraint fk_owner foreign key(owner_id) references users(id)
);

create table chains (
    id varchar not null unique,

    name varchar not null,

    primary key (id)
);

create type rpc_visibility as enum ('public', 'private');

create table rpcs (
    id serial,

    chain_id varchar not null,
    url varchar not null unique,
    visibility rpc_visibility not null default 'public',

    primary key (id),
    constraint fk_chain foreign key(chain_id) references chains(id)
);

create table groups_rpcs (
    group_id uuid not null unique,
    rpc_id int not null unique,
    enable boolean default TRUE,

    primary key (group_id, rpc_id),
    constraint fk_group foreign key(group_id) references groups(id),
    constraint fk_rpc foreign key(rpc_id) references rpcs(id)
);
