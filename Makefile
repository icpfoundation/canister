user := principal "dzhx6-f63tz-aslp6-xxyzd-pknwt-lxpho-q2wsx-pvwwd-v3nq6-75ek5-rqe"
userName := "test1"

groupId := 1
createTime := 10000
groupName := "test_group"
groupDescription := "test group"
visibility := variant {Private}
projects := vec {}
groupMemberName := "m1"
groupMemberAuthority :=  variant {Write}
groupMemberIdentity := $(user)
groupMembers := record{0 =  $(user);1 = record { name = $(groupMemberName);authority =$(groupMemberAuthority); identity = $(groupMemberIdentity)}}

projectMemberName := "member1"
projectMemberAuthority := variant {Write}
projectMemberIdentity := $(user)

projectId := 1
projectName := "test project"
projectDescription := "test project"
projectCreateBy := $(user)
projectCreateTime := 1000
projectGitRepoUrl := "*****.git"
projectVisibility := variant {Private}
projectInGroup := $(groupId)
projectMembers := record {0 = $(user); 1 = record {name = $(projectMemberName);authority = $(projectMemberAuthority);identity = $(projectMemberIdentity)}}
projectCanisters := vec {}

projectCanister := principal "ryjl3-tyaaa-aaaaa-aaaba-cai"




.PHONY : restart deploy set_controller get_status add_user get_user_info add_group remove_group add_project add_group_member remove_group_member
restart:
	dfx stop && dfx start --clean --background

deploy:
	dfx deploy
	
set_controller:
	dfx canister --wallet $$(dfx identity get-wallet) update-settings --all --controller  rrkah-fqaaa-aaaaa-aaaaq-cai

get_status:
	dfx canister call manage get_canister_status '(principal "rrkah-fqaaa-aaaaa-aaaaq-cai")'

add_user:
	dfx canister call manage add_user '("test1",variant { Public})'

get_user_info:
	dfx canister call manage get_user_info '($(user))'

add_group:
	dfx canister call manage add_group '(record {id = $(groupId); \
	create_time=$(createTime); \
	name=$(groupName); \
	description=$(groupDescription); \
	visibility=$(visibility); \
	projects = $(projects); \
	members = vec {$(groupMembers)}})'

remove_group:
	dfx canister call manage remove_group '($(groupId))'

add_project:
	dfx canister call manage add_project '($(groupId),\
	record {id=$(projectId); \
	name=$(projectName); \
	description=$(projectDescription); \
	create_by = $(projectCreateBy); \
	create_time = $(projectCreateTime); \
	git_repo_url = $(projectGitRepoUrl); \
	visibility = $(projectVisibility); \
	in_group = $(projectInGroup); \
	members = vec {$(projectMembers)}; \
	canisters = $(projectCanisters)})'

add_project_canister:
	dfx canister call manage add_project_canister '($(user),$(projectId),$(groupId),$(projectCanister))'

remove_project_canister:
	dfx canister call manage remove_project_canister '($(user),$(projectId),$(groupId),$(projectCanister))'

update_project_git_repo_url:
	dfx canister call manage update_project_git_repo_url  '($(user),$(projectId),$(groupId),"chaincloud.git")'

update_project_visibility:
	dfx canister call manage update_project_visibility  '($(user),$(projectId),$(groupId),variant {Public})'

update_project_description:
	dfx canister call manage update_project_description  '($(user),$(projectId),$(groupId),"canister management platform")'

remove_project:
	dfx canister call manage remove_project '($(groupId),$(projectId))'

add_group_member:
	dfx canister call manage add_group_member '($(groupId), \
	record {name=$(projectMemberName); \
	authority = $(projectMemberAuthority); \
	identity = $(projectMemberIdentity);})'

remove_group_member:
	dfx canister call manage remove_group_member '($(groupId),$(projectMemberIdentity))'




test:
	make restart \
	&& make deploy \
	&& make add_user \
	&& make add_group \
	&& make add_project \
	&& make add_project_canister \
	&& make update_project_git_repo_url \
	&& make update_project_visibility \
	&& make update_project_description \
	&& make add_group_member \
	&& make remove_group_member \

