-- Add down migration script here
drop table if exists groups_rpcs;
drop table if exists rpcs;
drop table if exists chains;
drop table if exists groups;

drop type if exists rpc_visiblity;

