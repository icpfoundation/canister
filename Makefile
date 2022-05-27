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
groupMemberAuthority :=  variant {Operational}
groupMemberIdentity := $(user)
groupMembers := record{0 =  $(user);1 = record { name = $(groupMemberName);authority =$(groupMemberAuthority); identity = $(groupMemberIdentity);join_time = 0}}

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
projectCanisterCycleFloor := 1000000000000
projectMembers := record {0 = $(user); 1 = record {name = $(projectMemberName);authority = $(projectMemberAuthority);identity = $(projectMemberIdentity);join_time = 0}}
projectCanisters := vec {}
projectType := variant {Wallet}
projectCanister := principal "rkp4c-7iaaa-aaaaa-aaaca-cai"

manageCanister := principal "rrkah-fqaaa-aaaaa-aaaaq-cai"
logCanister := principal "renrk-eyaaa-aaaaa-aaada-cai"
installCodeMode := variant { reinstall }
wasm := 

.PHONY : restart deploy set_controller get_status add_user get_user_info add_group remove_group add_project add_group_member remove_group_member
restart:
	dfx stop && dfx start --clean --background

dfxmange:
	dfx deploy manage \
	&& dfx deploy wallet \
	&& dfx deploy test_canister

dfxlogimage:
	dfx deploy image_store  --argument '($(manageCanister))' \
	&& dfx deploy canister_log  --argument '($(manageCanister))' \

updatelog:
	dfx canister call manage update_log_canister '($(logCanister))'

deploy:
	make dfxmange && make dfxlogimage && make updatelog

set_controller:
	dfx canister --wallet $$(dfx identity get-wallet) update-settings --all --controller rrkah-fqaaa-aaaaa-aaaaq-cai

get_canister_status:
	$(dfxManageCanister) get_canister_status '($(user),$(groupId),$(projectId),$(projectCanister))'

add_user:
	$(dfxManageCanister) add_user '("test1",variant { Public})'

get_user_info:
	$(dfxManageCanister) get_user_info '($(user))'

add_group:
	$(dfxManageCanister) add_group '($(user), record {id = $(groupId); \
	create_time=$(createTime); \
	name=$(groupName); \
	description=$(groupDescription); \
	visibility=$(visibility); \
	projects = $(projects); \
	members = vec {$(groupMembers)}})'

remove_group:
	$(dfxManageCanister) remove_group '($(user),$(groupId))'

add_project:
	$(dfxManageCanister) add_project '($(user),$(groupId),\
	record {id=$(projectId); \
	name=$(projectName); \
	description=$(projectDescription); \
	create_by = $(projectCreateBy); \
	create_time = $(projectCreateTime); \
	git_repo_url = $(projectGitRepoUrl); \
	visibility = $(projectVisibility); \
	in_group = $(projectInGroup); \
	members = vec {$(projectMembers)}; \
	canister_cycle_floor = $(projectCanisterCycleFloor); \
	canisters = $(projectCanisters); \
	function = $(projectType)})'


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

update_canister_cycle_floor:
	$(dfxManageCanister) update_canister_cycle_floor '($(user),$(groupId),$(projectId),10000000)'
	
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
	
get_group_member_info:
	$(dfxManageCanister) get_group_info  '($(user),$(groupId),$(projectMemberIdentity))'

visible_project:
	$(dfxManageCanister) visible_project



image_store:
	dfx canister call image_store image_store '(principal "r7inp-6aaaa-aaaaa-aaabq-cai",principal "r7inp-6aaaa-aaaaa-aaabq-cai", 1,vec {0;1;2;3;4;5;6;7;8;9;10;112})' \
	&& dfx canister call image_store image_store '(principal "r7inp-6aaaa-aaaaa-aaabq-cai", principal "r7inp-6aaaa-aaaaa-aaabq-cai", 2,vec {0;1;2;3;4;5;6;7;8;9;10;112;113;114})'

get_image:
	dfx canister call image_store get_image '(principal "r7inp-6aaaa-aaaaa-aaabq-cai", 1)' \
	&& dfx canister call image_store get_image '(principal "r7inp-6aaaa-aaaaa-aaabq-cai", 2)'

upgrade:
    dfx canister install --all --mode=upgrade


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


