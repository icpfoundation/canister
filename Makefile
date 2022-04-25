user := principal "dzhx6-f63tz-aslp6-xxyzd-pknwt-lxpho-q2wsx-pvwwd-v3nq6-75ek5-rqe"
dfxManageCanister := dfx canister call manage
userName := "test1"

groupId := 1
createTime := 10000
groupName := "test_group"
groupDescription := "test group"
visibility := variant {Public}
projects := vec {}
groupMemberName := "m1"
groupMemberAuthority :=  variant {Write}
groupMemberIdentity := $(user)
groupMembers := record{0 =  $(user);1 = record { name = $(groupMemberName);authority =$(groupMemberAuthority); identity = $(groupMemberIdentity)}}

projectMemberName := "member1"
projectMemberAuthority := variant {Operational}
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

projectCanister := principal "r7inp-6aaaa-aaaaa-aaabq-cai"


installCodeMode := variant { reinstall }
wasm := 

.PHONY : restart deploy set_controller get_status add_user get_user_info add_group remove_group add_project add_group_member remove_group_member
restart:
	dfx stop && dfx start --clean --background

deploy:
	dfx deploy
	
set_controller:
	dfx canister --wallet $$(dfx identity get-wallet) update-settings --all --controller  rrkah-fqaaa-aaaaa-aaaaq-cai

get_canister_status:
	$(dfxManageCanister) get_canister_status '($(user),$(groupId),$(projectId),$(projectCanister))'

add_user:
	$(dfxManageCanister) add_user '("test1",variant { Public})'

get_user_info:
	$(dfxManageCanister) get_user_info '($(user))'

add_group:
	$(dfxManageCanister) add_group '(record {id = $(groupId); \
	create_time=$(createTime); \
	name=$(groupName); \
	description=$(groupDescription); \
	visibility=$(visibility); \
	projects = $(projects); \
	members = vec {$(groupMembers)}})'

remove_group:
	$(dfxManageCanister) remove_group '($(groupId))'

add_project:
	$(dfxManageCanister) add_project '($(groupId),\
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


add_project_member:
	$(dfxManageCanister) add_project_member '($(user),\
	$(groupId), \
	$(projectId), \
	record {name=$(projectMemberName); \
	authority = $(projectMemberAuthority); \
	identity = $(projectMemberIdentity);})'

remove_project_member:
	$(dfxManageCanister) remove_project_member '($(user),\
	$(groupId), \
	$(projectId), \
	$(projectMemberIdentity))'


add_project_canister:
	$(dfxManageCanister) add_project_canister '($(user),$(groupId),$(projectId),$(projectCanister))'

remove_project_canister:
	$(dfxManageCanister) remove_project_canister '($(user),$(groupId),$(projectId),$(projectCanister))'

update_project_git_repo_url:
	$(dfxManageCanister) update_project_git_repo_url  '($(user),$(groupId),$(projectId),"chaincloud.git")'

update_project_visibility:
	$(dfxManageCanister) update_project_visibility  '($(user),$(groupId),$(projectId),variant {Public})'

update_project_description:
	$(dfxManageCanister) update_project_description  '($(user),$(groupId),$(projectId),"canister management platform")'

remove_project:
	$(dfxManageCanister) remove_project '($(groupId),$(projectId))'

add_group_member:
	$(dfxManageCanister) add_group_member '($(groupId), \
	record {name=$(projectMemberName); \
	authority = $(projectMemberAuthority); \
	identity = $(projectMemberIdentity);})'

remove_group_member:
	$(dfxManageCanister) remove_group_member '($(groupId),$(projectMemberIdentity))'

stop_project_canister:
	$(dfxManageCanister) stop_project_canister '($(user),$(groupId),$(projectId),$(projectCanister))'

start_project_canister:
	$(dfxManageCanister) start_project_canister '($(user),$(groupId),$(projectId),$(projectCanister))'

install_code:
	$(dfxManageCanister) install_code '($(user),$(groupId),$(projectId),$(projectCanister))'

get_project_info:
	$(dfxManageCanister) get_project_info  '($(user),$(groupId),$(projectId))'

get_group_info:
	$(dfxManageCanister) get_group_info  '($(user),$(groupId))'

test:
	make restart \
	&& make deploy \
	&& make set_controller \
	&& make add_user \
	&& make add_group \
	&& make add_project \
	&& make add_project_canister \
	&& make add_project_member \
	&& make get_project_info \
	&& make get_group_info \
	&& make get_user_info \
	&& make remove_project_member \
	&& make get_user_info \
	# && make stop_project_canister \
	# && make start_project_canister \
	# && make update_project_git_repo_url \
	# && make update_project_visibility \
	# && make update_project_description \
	# && make get_canister_status \
	# && make add_group_member \
	# && make remove_group_member	\


